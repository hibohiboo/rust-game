## 初回のみ

```
git config --local core.hooksPath .githooks && echo Changed hooks directory when npm install
```


## How to install

```sh
npm install
```

## How to run in debug mode

```sh
# Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
npm start
```

## How to build in release mode

```sh
# Builds the project and places it into the `dist` folder.
npm run build
```

## How to run unit tests

```sh
# Runs tests in Firefox
npm test -- --firefox

# Runs tests in Chrome
npm test -- --chrome

# Runs tests in Safari
npm test -- --safari
```

## What does each file do?

* `Cargo.toml` contains the standard Rust metadata. You put your Rust dependencies in here. You must change this file with your details (name, description, version, authors, categories)

* `package.json` contains the standard npm metadata. You put your JavaScript dependencies in here. You must change this file with your details (author, name, version)

* `webpack.config.js` contains the Webpack configuration. You shouldn't need to change this, unless you have very special needs.

* The `js` folder contains your JavaScript code (`index.js` is used to hook everything into Webpack, you don't need to change it).

* The `src` folder contains your Rust code.

* The `static` folder contains any files that you want copied as-is into the final build. It contains an `index.html` file which loads the `index.js` file.

* The `tests` folder contains your Rust unit tests.

## 最新のパッケージ
https://crates.io/search

## 素材

https://github.com/PacktPublishing/Game-Development-with-Rust-and-WebAssembly/wiki/Assets

## スプライト作成
https://www.codeandweb.com/texturepacker

## 最終
https://github.com/hibohiboo/Game-Development-with-Rust-and-WebAssembly

## linter

インストール
```
rustup component add clippy
```

実行

```
cargo clippy
```

## デプロイ
rust-toolchain.toml ... コンパイルに使用するRustのバージョンを指定

## Netlify
登録しておく。

### ログイン
```
npx netlify login
```
### 準備

```
npx netlify init --manual
```

```
npx netlify status
```


## アーキテクチャ

```mermaid
  info
```

```mermaid
graph TD;
    game-->engine;
    engine-->browser;
```

同じレイヤか、一段下のレイヤのものしか使えないとする