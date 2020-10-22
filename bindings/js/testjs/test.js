import { expect } from 'chai';
import { compute_root, create_proof, verify_proof } from 'mrklj';
import BLAKE2s from 'blake2s-js';
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
    const leaf_hashes = testleaves.map(blake2s);
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
    const leaf_hashes = testleaves.map(blake2s);
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
    const leaf_hashes = testleaves.map(blake2s);
    const lh_packed = pack32(leaf_hashes);
    const root = compute_root(lh_packed);
    const proofs = leaf_hashes.map((_, i) => create_proof(i, lh_packed));
    expect(root).to.deep.equal(new Uint8Array([
      129, 200, 233, 73, 31, 39, 141, 117, 148, 124, 248, 116, 3, 216, 180, 82,
      228, 244, 145, 31, 56, 205, 241, 85, 174, 94, 170, 12, 19, 228, 198, 164
    ]));
    expect(proofs).to.deep.equal([
      [
        {
          Right: [
            225, 249, 175, 63, 145, 98, 79, 188, 171, 95, 57, 196, 203, 174, 70, 222,
            183, 145, 208, 72, 39, 24, 74, 24, 80, 10, 37, 96, 240, 40, 239, 65
          ]
        }, {
          Right: [
            95, 190, 36, 79, 242, 19, 244, 31, 231, 117, 175, 188, 226, 50, 8, 114,
            157, 188, 100, 236, 215, 125, 176, 182, 114, 7, 145, 240, 156, 182, 75, 56
          ]
        }
      ],
      [
        {
          Left: [
            119, 194, 250, 6, 9, 173, 18, 24, 21, 138, 21, 10, 21, 85, 204, 7,
            157, 145, 77, 37, 210, 161, 133, 12, 155, 14, 102, 64, 104, 17, 196, 197
          ]
        }, {
          Right: [
            95, 190, 36, 79, 242, 19, 244, 31, 231, 117, 175, 188, 226, 50, 8, 114,
            157, 188, 100, 236, 215, 125, 176, 182, 114, 7, 145, 240, 156, 182, 75, 56
          ]
        }
      ],
      [
        {
          Left: [
            38, 175, 220, 15, 117, 10, 163, 114, 151, 247, 152, 47, 235, 153, 30, 18,
            17, 185, 255, 198, 26, 98, 234, 166, 105, 186, 89, 25, 112, 127, 165, 85
          ]
        }
      ]
    ]);
  }],
]);

function utf8(str) {
  return new TextEncoder("utf-8").encode(str);
}

// hash a byte array using blake2s-256
function blake2s(bs) {
  let h = new BLAKE2s();
  h.update(bs);
  return h.digest();
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
