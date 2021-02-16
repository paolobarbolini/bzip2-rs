# 0.1.2 (February 16, 2021)

### Fixed

- block: fix harmless overflow in huffman table bit run length decoder ([101960fd])

### Improved

- huffman: optimize HuffmanTree::decode ([115c477d])
- bitreader: implement CachedBitReader and use it for huffman decoding ([99316d42]) ([74ffefc6])

### Changed

- cargo: enable the nightly feature of crc32fast when we are in nightly ([6330fcee])

### Documented

- docs: fix Decoder intra-doc link ([7c7025fa])

[101960fd]: https://github.com/paolobarbolini/bzip2-rs/commit/101960fdd8d7f1520a6a33bfc7970e971bb0ebe5
[115c477d]: https://github.com/paolobarbolini/bzip2-rs/commit/115c477de7d89a74c044e4b6cccae34628054436
[99316d42]: https://github.com/paolobarbolini/bzip2-rs/commit/99316d423df70533d63a9a7f51c09f4a3c6fcbce
[74ffefc6]: https://github.com/paolobarbolini/bzip2-rs/commit/74ffefc67e4ab57b0fd5c8548cd768866edd54c8
[6330fcee]: https://github.com/paolobarbolini/bzip2-rs/commit/6330fceea0dc8413843d1b0e355e720e5d13e953
[7c7025fa]: https://github.com/paolobarbolini/bzip2-rs/commit/7c7025fa8af9a313404df789313539e77ea93f89

# 0.1.1 (January 31, 2021)

### Added

- fuzz: add initial fuzzing configuration ([3f07135])

### Fixed

- decoder: avoid hanging `DecoderReader` when the supplied `Read` is empty ([9d24208]).

[3f07135]: https://github.com/paolobarbolini/bzip2-rs/commit/3f0713598236e0bf3b8721e8a052ef638fb7b94c
[9d24208]: https://github.com/paolobarbolini/bzip2-rs/commit/9d24208e7c953cd510239340f499e47f5b70b305

# 0.1.0 (January 30, 2021)

First public release.
