import { expect } from 'chai';
import { compute_root, create_proof, verify_proof, construct } from 'mrklj';
import { blake2b } from 'blakejs';
import assert from 'assert';
import deepEqual from 'deep-equal';

// poor man's replacement for jest because making jest work with webpack+wasm is problematic
function tests(tests) {
  let red = '\x1b[31m';
  let green = '\x1b[32m';
  let reset = '\x1b[0m';

  let stats = [];
  for (let [name, cb] of tests) {
    let passed;
    try {
      cb();
      console.log(green + '✓ ', name, 'passed', reset);
      passed = true;
    } catch (e) {
      console.error(red + '❌', name, reset);
      console.log(e);
      passed = false;
    }
    stats.push(passed);
  }
  let passed_count = stats.filter(a => a).length;
  console.log(`${passed_count}/${tests.length} tests passed`);
  process.exit(passed_count === tests.length ? 0 : 1);
}

tests([
  ['happy path', () => {
    const testleaves = [
      utf8("hello"),
      utf8("I exist"),
      utf8("I exist"),
      utf8("srsly"),
      utf8("Odd number of leaves means an unbalanced merkle tree but it should still work."),
    ];
    const leaf_hashes = testleaves.map(blake2b256);
    const lh_packed = pack32(leaf_hashes);
    const root = compute_root(lh_packed);
    const proofs = leaf_hashes.map((_, i) => create_proof(i, lh_packed));
    for (const [lh, proof] of zip(leaf_hashes, proofs)) {
      expect(verify_proof(lh, proof)).to.deep.eq(root);
    }
  }],
  ['failing path', () => {
    const testleaves = [
      utf8("1"),
      utf8("2"),
      utf8("3"),
      utf8("4"),
    ];
    const leaf_hashes = testleaves.map(blake2b256);
    const lh_packed = pack32(leaf_hashes);
    const root = compute_root(lh_packed);
    const proofs = leaf_hashes.map((_, i) => create_proof(i, lh_packed));
    for (const [proof, i] of enumerate(proofs)) {
      for (const [lh, j] of enumerate(leaf_hashes)) {
        expect(deepEqual(verify_proof(lh, proof), root)).to.equal(i === j);
      }
    }
  }],
  ['expected output', () => {
    const testleaves = [
      utf8("1"),
      utf8("2"),
      utf8("3"),
    ];
    const leaf_hashes = testleaves.map(blake2b256);
    const lh_packed = pack32(leaf_hashes);
    const root = compute_root(lh_packed);
    const proofs = leaf_hashes.map((_, i) => create_proof(i, lh_packed));
    expect(root).to.deep.equal(new Uint8Array([
      139, 45, 145, 103, 46, 124, 238, 23, 51, 84, 193, 22, 201, 120, 11, 88, 127, 192, 49, 28, 165,
      31, 74, 143, 112, 84, 7, 204, 228, 5, 214, 236
    ]));
    expect(proofs).to.deep.equal([
      [
        {
          Right: [
            151, 61, 106, 145, 33, 102, 201, 84, 145, 96, 87, 235, 106, 7, 211, 232, 191, 69, 31,
            204, 170, 186, 139, 247, 255, 246, 44, 174, 42, 19, 199, 64
          ]
        },
        {
          Right: [
            100, 0, 145, 252, 75, 92, 47, 44, 17, 241, 128, 30, 80, 82, 6, 53, 157, 139, 2, 153,
            84, 121, 13, 220, 10, 183, 200, 148, 56, 181, 136, 118
          ]
        }
      ],
      [
        {
          Left: [
            206, 225, 179, 65, 151, 130, 173, 146, 236, 45, 255, 237, 109, 63, 54, 203, 153, 244,
            190, 143, 114, 128, 255, 227, 168, 116, 176, 43, 255, 169, 168, 97
          ]
        },
        {
          Right: [
            100, 0, 145, 252, 75, 92, 47, 44, 17, 241, 128, 30, 80, 82, 6, 53, 157, 139, 2, 153, 84,
            121, 13, 220, 10, 183, 200, 148, 56, 181, 136, 118
          ]
        }
      ],
      [
        {
          Left: [
            19, 242, 94, 150, 193, 248, 24, 234, 126, 25, 62, 124, 156, 160, 118, 185, 207, 204,
            201, 5, 4, 114, 42, 54, 124, 47, 119, 252, 4, 33, 162, 189
          ]
        }
      ]
    ]);
  }],
  ['construct() has the same result as compute_root() together with create_proof()', () => {
    for (let i = 1; i < 30; i++) {
      const leaf_hashes = randoLeaves(i);
      const lh_packed = pack32(leaf_hashes);
      const root = compute_root(lh_packed);
      const proofs = leaf_hashes.map((_, i) => create_proof(i, lh_packed));
      const [otherRoot, otherProofs] = construct(lh_packed);
      expect([[...root], proofs]).to.deep.eq([otherRoot, otherProofs]);
    }
  }],
]);

function randoLeaves(count) {
  return Array(count).fill(undefined).map(() => randoHash());
}

function randoHash() {
  return new Uint8Array(Array(32).fill(undefined).map(() => Math.floor(Math.random() * 256)));
}

function utf8(str) {
  return new TextEncoder("utf-8").encode(str);
}

// hash a byte array using blake2b-256
function blake2b256(bs) {
  assert(bs instanceof Uint8Array);
  return blake2b(bs, undefined, 32);
}

// pack a list of hashed leaves into a single byte array
function pack32(leaves) {
  for (const leaf of leaves) {
    assert(leaf instanceof Uint8Array);
    assert(leaf.length == 32);
  }
  let ret = new Uint8Array(leaves.map(a => [...a]).flat());
  assert(ret.length === leaves.length * 32);
  return ret;
}

function zip(a, b) {
  assert(a.length === b.length);
  return a.map((_, i) => [a[i], b[i]]);
}

function enumerate(ls) {
  return ls.map((l, i) => [l, i]);
}
