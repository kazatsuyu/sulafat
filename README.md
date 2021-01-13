# Sulafat

## これは何？

仮想DOMベースのGUIフレームワーク（を目指している何か）

## 特徴

* Rust製（一部TypeScript)
* ブラウザ上でWASMで動く（一応、手元のテストでは動いた）
* ネイティブでも動……かない（動くようになるといいなあ）
* The Elm Architectureの真似

まだ全く何も完成していないしRust製Webフレームワーク使いたい人は[yew](https://github.com/yewstack/yew)とか使うといいと思うよ！

## 一応何か動くことを確かめたい人へ

前提

* Rust(nightly)
    * `feature = "nightly-fetures"`を使わなければnightlyじゃなくても動くようにはしているつもりだけど確認していない
* Node.js
* Yarn（もしくはNpm）
* [simple-http-server](https://crates.io/crates/simple-http-server)
    * 単に静的HTTPサーバーがあれば良いので他の手段でも良い。`yarn start`コマンドがこれを呼ぶようになっている

[runtime-web](./runtime-web)ディレクトリ（完成した暁にはWeb用ランタイムになる筈だが今は手動テスト実験場）に移動して

```bash
yarn install
yarn build
yarn start
```

を実行。ブラウザで[http://localhost:8000/tests/index.html](http://localhost:8000/tests/index.html)を開く。
