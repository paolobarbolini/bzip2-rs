use tinyvec::ArrayVec;

use crate::bitreader::BitReader;

pub struct HuffmanTree {
    nodes: [HuffmanNode; 258],
}

#[derive(Default, Copy, Clone)]
struct HuffmanNode {
    // [right, left]
    right_left: [HuffmanNodeState; 2],
}

#[derive(Copy, Clone)]
enum HuffmanNodeState {
    Next(u16),
    Done(u16),
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

                code = code.wrapping_add(1 << (32 - length));
                c
            })
            .collect::<ArrayVec<[HuffmanCode; crate::LEN_258]>>();

        codes.sort_unstable_by(|a, b| a.code.cmp(&b.code));

        let mut this = HuffmanTree::default();
        Self::build_huffman_node(&mut this.nodes[..codes.len()], &mut 0, &codes, 0)?;
        Ok(this)
    }

    pub fn decode(&self, bits_iter: &mut BitReader<'_>) -> Option<u16> {
        let mut node = &self.nodes[0];

        for bit in bits_iter {
            let val = &node.right_left[usize::from(bit)];
            match val {
                HuffmanNodeState::Next(node_index) => node = &self.nodes[usize::from(*node_index)],
                HuffmanNodeState::Done(value) => return Some(*value),
            };
        }

        None
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
                node.right_left[1] = HuffmanNodeState::Done(left[0].value);
            } else {
                let val = Self::build_huffman_node(nodes, next_node, left, level + 1)?;

                let node = &mut nodes[node_index];
                node.right_left[1] = HuffmanNodeState::Next(val);
            }

            if right.len() == 1 {
                let node = &mut nodes[node_index];
                node.right_left[0] = HuffmanNodeState::Done(right[0].value);
            } else {
                let val = Self::build_huffman_node(nodes, next_node, right, level + 1)?;

                let node = &mut nodes[node_index];
                node.right_left[0] = HuffmanNodeState::Next(val);
            }

            Ok(node_index as u16)
        }
    }
}

impl Default for HuffmanTree {
    fn default() -> Self {
        Self {
            nodes: [HuffmanNode::default(); 258],
        }
    }
}

impl Default for HuffmanNodeState {
    fn default() -> Self {
        HuffmanNodeState::Next(0)
    }
}
