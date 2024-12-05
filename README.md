# FileHash

<HEADER>
    - no of total buckets
    - no of empty buckets
<INDICES>
    - array of u8 (0 & 1) indicating if bucket is occupied of not
<DATA>
  - <INDEX>(0, 1, 2..n) | <KEY> | <VALUE>
  - 2 bytes | 16 bytes | 108 bytes -> 128 bytes