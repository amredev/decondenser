# decondenser website

This folder contains the source code and markdown files that comprise the [decondenser website](https://decondenser.dev).

The website is built using [VitePress](https://vitepress.dev/). It's a simple and elegant framework for static websites that hides a lot of complexity from you. You don't need to be a TypeScript expert let alone a Vue expert to get around this directory.

## Local server with hot reloading

If you want to make changes to the website, you need to be able to preview the updated web pages locally.

First, make sure you have [NodeJS installed](https://nodejs.org/en/download/package-manager).

Then install the dependencies. This command must be run from the root of the repository.

```bash
npm install --workspace decondenser-website
```

Now you can run the local server with the following command:

```bash
npm run website:dev
```

Consult [VitePress](https://vitepress.dev/) docs for help if you need any.

#### License

<sup>
Licensed under either of <a href="https://github.com/amredev/decondenser/blob/master/LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="https://github.com/amredev/decondenser/blob/master/LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
