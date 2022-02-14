name: CI

env:
  DEBUG: 'napi:*'
  APP_NAME: '{{ binary_name }}'
  MACOSX_DEPLOYMENT_TARGET: '10.13'

on:
  push:
    branches:
      - main
    tags-ignore:
      - '**'
    paths-ignore:
      - '**/*.md'
      - 'LICENSE'
      - '**/*.gitignore'
      - '.editorconfig'
      - 'docs/**'
  pull_request:

jobs:
  build:
    if: "!contains(github.event.head_commit.message, 'skip ci')"

    strategy:
      fail-fast: false
      matrix:
        settings: {% for (target, github_workflow_config) in targets %}
          - target: {{ target.triple }}
            host: {{ github_workflow_config.host }}{% if github_workflow_config.docker %}
            docker: $DOCKER_REGISTRY_URL/{{ github_workflow_config.docker }}{% endif %}{% if github_workflow_config.setup %}
            setup: |{% for line in github_workflow_config.setup %}
              {{line}}{% endfor%}{% endif %}{% endfor %}

    name: stable - ${{ "{{" }} matrix.settings.target {{ "}}" }} - node@16
    runs-on: ${{ "{{" }} matrix.settings.host {{ "}}" }}
    steps:
      - uses: actions/checkout@v3

      - name: Setup node
        uses: actions/setup-node@v3
        if: ${{ "{{" }} !matrix.settings.docker {{ "}}" }}
        with:
          node-version: 16
          check-latest: true
          cache: yarn

      - name: Install
        uses: actions-rs/toolchain@v1
        if: ${{ "{{" }} !matrix.settings.docker {{ "}}" }}
        with:
          profile: minimal
          override: true
          toolchain: stable
          target: ${{ "{{" }} matrix.settings.target {{ "}}" }}

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            .cargo-cache/registry/index/
            .cargo-cache/registry/cache/
            .cargo-cache/git/db/
            target/
          key: ${{ "{{" }} matrix.settings.target {{ "}}" }}-cargo-${{ "{{" }} matrix.settings.host {{ "}}" }}

      - name: Cache NPM dependencies
        uses: actions/cache@v3
        with:
          path: .yarn/cache
          key: npm-cache-build-${{ "{{" }} matrix.settings.target {{ "}}" }}-node@16

      - name: Setup toolchain
        run: ${{ "{{" }} matrix.settings.setup {{ "}}" }}
        if: ${{ "{{" }} matrix.settings.setup {{ "}}" }}
        shell: bash

      - name: Setup node x86
        if: matrix.settings.target == 'i686-pc-windows-msvc'
        run: yarn config set supportedArchitectures.cpu "ia32"
        shell: bash

      - name: Install dependencies
        run: yarn install

      - name: Setup node x86
        uses: actions/setup-node@v3
        if: matrix.settings.target == 'i686-pc-windows-msvc'
        with:
          node-version: 16
          check-latest: true
          cache: yarn
          architecture: x86

      - name: Build in docker
        uses: addnab/docker-run-action@v3
        if: ${{ "{{" }} matrix.settings.docker {{ "}}" }} 
        with:
          image: ${{ "{{" }} matrix.settings.docker {{ "}}" }} 
          options: --user 0:0 -v ${{ "{{" }} github.workspace {{ "}}" }}/.cargo-cache/git/db:/root/.cargo/git/db -v ${{ "{{" }} github.workspace {{ "}}" }}/.cargo/registry/cache:/root/.cargo/registry/cache -v ${{ "{{" }} github.workspace {{ "}}" }}/.cargo/registry/index:/root/.cargo/registry/index -v ${{ "{{" }} github.workspace {{ "}}" }}:/build -w /build
          run: |
            cargo install napi-cli
            ${{ "{{" }} matrix.settings.setup {{ "}}" }} 
            napi build --target ${{ "{{" }} matrix.settings.target {{ "}}" }}

      - name: Build
        shell: bash
        if: ${{ "{{" }} !matrix.settings.docker {{ "}}" }} 
        run: |
          cargo install napi-cli
          ${{ "{{" }} matrix.settings.setup {{ "}}" }} 
          napi build --target ${{ "{{" }} matrix.settings.target {{ "}}" }}

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: bindings-${{ "{{" }} matrix.settings.target {{ "}}" }}
          path: ${{ "{{" }} env.APP_NAME  {{ "}}" }}.*.node
          if-no-files-found: error
  
  {% for (target, github_workflow_config) in targets %}
  test-{{ target.triple }}:
    name: Test bindings on {{target.triple}} - node@-${{ "{{" }} matrix.node {{ "}}" }}
    needs:
      - build
    strategy:
      fail-fast: false
      matrix:
        node: ['14', '16', '18']
    runs-on: {{ github_workflow_config.host }}
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup node
        uses: actions/setup-node@v3
        with:
          node-version: ${{ "{{" }} matrix.node {{ "}}" }}
          check-latest: true
          cache: yarn
      
      - name: Install dependencies
        run: yarn install

      - name: Download artifacts
        uses: actions/download-artifact@v2
        with:
          name: bindings-${{ "{{" }} matrix.settings.target {{ "}}" }}
          path: .

      - name: List packages
        run: ls -R .
        shell: bash

      - name: Test bindings
        run: docker run --rm -v $(pwd):/build -w /build node:${{ "{{" }} matrix.node {{ "}}" }}-slim yarn test
  {% endfor %}

  publish:
    name: Publish
    runs-on: ubuntu-latest
    needs:
      - build
      {% for (target, _) in targets %}- test-{{ target.triple }}
      {% endfor %}
    steps:
      - uses: actions/checkout@v2
      - name: Setup node
        uses: actions/setup-node@v2
        with:
          node-version: 16
          check-latest: true
          cache: 'yarn'
      - name: 'Install dependencies'
        run: yarn install

      - name: Download all artifacts
        uses: actions/download-artifact@v2
        with:
          path: artifacts

      - name: Move artifacts
        run: yarn artifacts

      - name: List packages
        run: ls -R ./npm
        shell: bash

      - name: Publish
        run: |
          if git log -1 --pretty=%B | grep "^[0-9]\\+\\.[0-9]\\+\\.[0-9]\\+$";
          then
            echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" >> ~/.npmrc
            npm publish --access public
          elif git log -1 --pretty=%B | grep "^[0-9]\\+\\.[0-9]\\+\\.[0-9]\\+";
          then
            echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" >> ~/.npmrc
            npm publish --tag next --access public
          else
            echo "Not a release, skipping publish"
          fi
        env:
          GITHUB_TOKEN: ${{ "{{" }} secrets.GITHUB_TOKEN {{ "}}" }}
          NPM_TOKEN: ${{ "{{" }} secrets.NPM_TOKEN {{ "}}" }}
