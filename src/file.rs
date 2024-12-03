// parts/partNo (0[no-parts], 1..n[numbers], -1[end]), key, value
// maxSize for keys (should fit inside a single block) and values (should fit inside 256 blocks)

// or generate a hashKey for the key so it can be of any length and also save some space

/*
 * # File
 * 
 * File to be stored on device for hash table
 * 
 * ## Working
 * 
 * - K-V pairs are stored in bytes
 * - Pre-Fill the file w/ null values
 * - Pre-defined bucket size for each item or pair
 * - For large values, they must be distributed 
 *   across multiple buckets
 * 
 * ## Structure
 * 
 * It should contain,
 * 
 * - key
 * - value
 * - 0 if no parts, 1..n for parts, -1 for last part
 * 
 */
