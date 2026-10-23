# January compressor compatibility fixture

Captured before the indexed-compressor hash/preallocation optimization from the already verified deterministic seed7 bot-century checkpoint `.blue-check/expansion/bot-river-century-rc.bcsave`.

- Source frame SHA256: `d120bb4b96d9c27d978ae4ed41300bcbf30a2382ad7bcac5d094ce88934efc62`.
- `january-government-reference.lz` is the actual saved government byte stream,26276 bytes, SHA256 `593aa0f9068c88e572ae3475e06cd0e3093156a10d5091d5a726efa072a93623`.
- The185811-byte JSON file is that same archived government decoded by the existing literal/back-reference format. It contains generated town test data, not player or session information.

The private codec test requires the new compressor to reproduce these exact bytes and the decoder to recover the exact JSON. Hash-map bucket choice and allocation changes may not alter the codec or its match choices. This fixture is separate from the released v1/v2 framed save migration fixtures.
