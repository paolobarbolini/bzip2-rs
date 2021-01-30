use tinyvec::ArrayVec;

use crate::bitreader::BitReader;

const INVALID_NODE_VALUE: u16 = 0xffff;

#[derive(Default)]
pub struct HuffmanTree {
    nodes: ArrayVec<[HuffmanNode; crate::LEN_258]>,
}

#[derive(Default, Clone)]
struct HuffmanNode {
    left: u16,
    right: u16,
    left_value: u16,
    right_value: u16,
}

#[derive(Default)]
struct LengthPair {
    value: u16,
    length: u8,
}

#[derive(Default)]
struct HuffmanCode {
    code: u32,
    value: u16,
}

impl HuffmanTree {
    pub fn new(lengths: &[u8]) -> Result<Self, &'static str> {
        assert!(lengths.len() >= 2, "too few symbols");

        let mut pairs = lengths
            .iter()
            .enumerate()
            .map(|(i, &length)| LengthPair {
                value: i as u16,
                length,
            })
            .collect::<ArrayVec<[LengthPair; crate::LEN_258]>>();

        pairs.sort_unstable_by(|a, b| {
            a.length
                .cmp(&b.length)
                .then_with(|| a.value.cmp(&b.value))
                .reverse()
        });

        let mut code = 0u32;
        let mut length = 32u8;

        let mut codes = pairs
            .into_iter()
            .map(|pair| {
                length = length.min(pair.length);

                let c = HuffmanCode {
                    code,
                    value: pair.value,
                };

                code = code.overflowing_add(1 << (32 - length)).0;
                c
            })
            .collect::<ArrayVec<[HuffmanCode; crate::LEN_258]>>();

        codes.sort_unstable_by(|a, b| a.code.cmp(&b.code));

        let mut nodes = ArrayVec::new();
        nodes.set_len(codes.len());
        Self::build_huffman_node(&mut nodes, &mut 0, &codes, 0)?;

        Ok(Self { nodes })
    }

    pub fn decode(&self, buf: &mut BitReader<'_>) -> Result<u16, &'static str> {
        let mut node_index = 0u16;

        loop {
            let node = &self.nodes[node_index as usize];

            let bit = buf.read_bool().ok_or("huffman bitstream truncated")?;

            node_index = if bit { node.left } else { node.right };

            if node_index == INVALID_NODE_VALUE {
                return Ok(if bit {
                    node.left_value
                } else {
                    node.right_value
                });
            }
        }
    }

    fn build_huffman_node(
        nodes: &mut [HuffmanNode],
        next_node: &mut usize,
        codes: &[HuffmanCode],
        level: u32,
    ) -> Result<u16, &'static str> {
        let test = 1u32 << (31 - level);

        let first_right_index = codes
            .iter()
            .enumerate()
            .find(|(_, code)| code.code & test != 0)
            .map_or(codes.len(), |(i, _)| i);

        let left = &codes[..first_right_index];
        let right = &codes[first_right_index..];

        if left.is_empty() || right.is_empty() {
            if codes.len() < 2 {
                return Err("empty huffman tree");
            }

            if level == 31 {
                return Err("equal symbols in huffman tree");
            }

            if left.is_empty() {
                Self::build_huffman_node(nodes, next_node, right, level + 1)
            } else {
                Self::build_huffman_node(nodes, next_node, left, level + 1)
            }
        } else {
            let node_index = *next_node;
            // remove downstream bounds checking
            assert!(node_index < nodes.len());
            *next_node += 1;

            if left.len() == 1 {
                let node = &mut nodes[node_index];
                node.left = INVALID_NODE_VALUE;
                node.left_value = left[0].value;
            } else {
                let val = Self::build_huffman_node(nodes, next_node, left, level + 1)?;

                let node = &mut nodes[node_index];
                node.left = val;
            }

            if right.len() == 1 {
                let node = &mut nodes[node_index];
                node.right = INVALID_NODE_VALUE;
                node.right_value = right[0].value;
            } else {
                let val = Self::build_huffman_node(nodes, next_node, right, level + 1)?;

                let node = &mut nodes[node_index];
                node.right = val;
            }

            Ok(node_index as u16)
        }
    }
}
