/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  // By default, Docusaurus generates a sidebar from the docs folder structure
  // defaultSidebar: [{type: 'autogenerated', dirName: '.', }],
  aptosSidebar: [
    {
      type: "html",
      value: "Start",
      className: "sidebar-title",
    },
    {
      type: "category",
      label: "Read the Aptos White Paper",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "aptos-white-paper/index" },
      items: ["aptos-white-paper/in-korean"],
    },
    {
      type: "category",
      label: "Learn about Aptos",
      link: { type: "doc", id: "concepts/index" },
      collapsible: true,
      collapsed: true,
      items: [
        {
          type: "category",
          label: "Aptos Blockchain Deep Dive",
          link: { type: "doc", id: "concepts/blockchain" },
          collapsible: true,
          collapsed: true,
          items: ["concepts/validator-nodes", "concepts/fullnodes", "concepts/node-networks-sync"],
        },
        "concepts/accounts",
        "concepts/resources",
        "concepts/events",
        "concepts/txns-states",
        "concepts/gas-txn-fee",
        "concepts/base-gas",
        "concepts/blocks",
        {
          type: "category",
          label: "Staking",
          link: { type: "doc", id: "concepts/staking" },
          collapsible: true,
          collapsed: true,
          items: ["concepts/delegated-staking"],
        },
        "concepts/governance",
      ],
    },
    "guides/explore-aptos",
    {
      type: "category",
      label: "Latest Releases",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "releases/index" },
      items: [
        {
          type: "category",
          label: "Node and Framework Release",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "releases/mainnet-release" },
          items: ["releases/mainnet-release", "releases/testnet-release", "releases/devnet-release"],
        },
        "releases/cli-release",
        "releases/sdk-release",
      ],
    },
    "nodes/deployments",
    "guides/system-integrators-guide",
  ],
  appSidebar: [
    {
      type: "html",
      value: "Build",
      className: "sidebar-title",
    },
    {
      type: "category",
      label: "Start with Onboarding Tutorials",
      link: { type: "doc", id: "tutorials/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "tutorials/first-transaction",
        "tutorials/your-first-nft",
        "tutorials/first-coin",
        "tutorials/first-fungible-asset",
        "tutorials/first-move-module",
        "tutorials/first-dapp",
        "tutorials/first-multisig",
      ],
    },
    {
      type: "category",
      label: "Learn the Move Language",
      link: { type: "doc", id: "move/move-on-aptos" },
      collapsible: true,
      collapsed: true,
      items: [
        {
          type: "category",
          label: "Move on Aptos",
          link: { type: "doc", id: "move/move-on-aptos" },
          collapsible: true,
          collapsed: true,
          items: [
            "move/move-on-aptos/resource-accounts",
            "move/move-on-aptos/modules-on-aptos",
            "move/move-on-aptos/move-scripts",
            "move/move-on-aptos/cli",
            "move/move-on-aptos/cryptography",
          ],
        },
        {
          type: "category",
          label: "The Move Book",
          link: { type: "doc", id: "move/book/SUMMARY" },
          collapsible: true,
          collapsed: true,
          items: [
            {
              type: "category",
              label: "Getting Started",
              collapsible: true,
              collapsed: true,
              items: ["move/book/introduction", "move/book/modules-and-scripts"],
            },
            {
              type: "category",
              label: "Primitive Types",
              collapsible: true,
              collapsed: true,
              items: [
                "move/book/creating-coins",
                "move/book/integers",
                "move/book/bool",
                "move/book/address",
                "move/book/vector",
                "move/book/signer",
                "move/book/references",
                "move/book/tuples",
              ],
            },
            {
              type: "category",
              label: "Basic Concepts",
              collapsible: true,
              collapsed: true,
              items: [
                "move/book/variables",
                "move/book/equality",
                "move/book/abort-and-assert",
                "move/book/conditionals",
                "move/book/loops",
                "move/book/functions",
                "move/book/structs-and-resources",
                "move/book/constants",
                "move/book/generics",
                "move/book/abilities",
                "move/book/uses",
                "move/book/friends",
                "move/book/packages",
                "move/book/package-upgrades",
                "move/book/unit-testing",
              ],
            },
            {
              type: "category",
              label: "Global Storage",
              collapsible: true,
              collapsed: true,
              items: ["move/book/global-storage-structure", "move/book/global-storage-operators"],
            },
            {
              type: "category",
              label: "Reference",
              collapsible: true,
              collapsed: true,
              items: ["move/book/standard-library", "move/book/coding-conventions"],
            },
          ],
        },
        {
          type: "category",
          label: "The Move Prover Book",
          link: { type: "doc", id: "move/prover/index" },
          collapsible: true,
          collapsed: true,
          items: ["move/prover/prover-guide", "move/prover/spec-lang", "move/prover/supporting-resources"],
        },
      ],
    },
    {
      type: "category",
      label: "Embrace the Aptos Standards",
      link: { type: "doc", id: "standards/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "standards/aptos-object",
        "standards/aptos-coin",
        "standards/fungible-asset",
        "standards/digital-asset",
        "standards/aptos-token",
        "standards/wallets",
      ],
    },
    {
      type: "category",
      label: "Integrate with Aptos",
      link: { type: "doc", id: "integration/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "integration/aptos-apis",
        {
          type: "category",
          label: "Integrate with Wallets",
          link: { type: "doc", id: "integration/wallet-adapter-concept" },
          collapsible: true,
          collapsed: true,
          items: ["integration/wallet-adapter-for-dapp", "integration/wallet-adapter-for-wallets"],
        },
        "integration/sign-a-transaction",
        "integration/aptos-names-service-package",
        "integration/handle-aptos-errors",
      ],
    },
    {
      type: "category",
      label: "Configure your Environment",
      collapsible: true,
      collapsed: true,
      link: {
        type: "generated-index",
        title: "Configure your Environment",
        description: "Prepare your development environment.",
        slug: "/category/environment",
      },
      items: [
        {
          type: "category",
          label: "Aptos CLI",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "tools/aptos-cli/index" },
          items: [
            {
              type: "category",
              label: "Install the Aptos CLI",
              link: { type: "doc", id: "tools/aptos-cli/install-cli/index" },
              collapsible: true,
              collapsed: true,
              items: [
                "tools/aptos-cli/install-cli/automated-install",
                "tools/aptos-cli/install-cli/download-cli-binaries",
                "tools/aptos-cli/install-cli/install-from-brew",
                "tools/aptos-cli/install-cli/build-from-source",
                "tools/aptos-cli/install-cli/install-move-prover",
              ],
            },
            {
              type: "category",
              label: "Use Aptos CLI",
              link: { type: "doc", id: "tools/aptos-cli/use-cli/use-aptos-cli" },
              collapsible: true,
              collapsed: true,
              items: [
                "tools/aptos-cli/use-cli/cli-configuration",
                "tools/aptos-cli/use-cli/cli-account",
                "tools/aptos-cli/use-cli/cli-key",
                "tools/aptos-cli/use-cli/cli-node",
                "tools/aptos-cli/use-cli/cli-genesis",
                "tools/aptos-cli/use-cli/use-aptos-ledger",
              ],
            },
          ],
        },
        {
          type: "category",
          label: "Aptos SDKs",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "sdks/index" },
          items: [
            {
              type: "category",
              label: "TypeScript SDK",
              link: { type: "doc", id: "sdks/ts-sdk/index" },
              collapsible: true,
              collapsed: true,
              items: [
                "sdks/ts-sdk/typescript-sdk-overview",
                {
                  type: "category",
                  label: "API Client Layer",
                  link: { type: "doc", id: "sdks/ts-sdk/sdk-client-layer" },
                  collapsible: true,
                  collapsed: true,
                  items: ["sdks/ts-sdk/aptos-client", "sdks/ts-sdk/indexer-client"],
                },
                "sdks/ts-sdk/sdk-core-layer",
                "sdks/ts-sdk/sdk-plugins-layer",
                "sdks/ts-sdk/sdk-tests",
              ],
            },
            "sdks/python-sdk",
            "sdks/rust-sdk",
            "sdks/unity-sdk",
          ],
        },
        "guides/building-from-source",
      ],
    },
    {
      type: "category",
      label: "Create NFTs on Aptos",
      collapsible: true,
      collapsed: true,
      link: {
        type: "generated-index",
        title: "Create Tokens on Aptos",
        description: "Learn the various ways to mint and exchange tokens.",
        slug: "/category/nft",
        keywords: ["nft"],
      },
      items: ["guides/nfts/aptos-token-comparison", "guides/nfts/mint-nft-cli", "guides/nfts/mint-onchain-data"],
    },
    {
      type: "category",
      label: "Examples",
      collapsible: true,
      collapsed: true,
      link: {
        type: "generated-index",
        title: "Examples",
        description: "Examples for all the various concepts and tooling used to build on Aptos.",
        slug: "/category/examples",
        keywords: ["examples"],
      },
      items: ["guides/account-management/key-rotation"],
    },
    {
      type: "category",
      label: "Build E2E Dapp with Aptos",
      link: { type: "doc", id: "tutorials/build-e2e-dapp/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "tutorials/build-e2e-dapp/create-a-smart-contract",
        "tutorials/build-e2e-dapp/set-up-react-app",
        "tutorials/build-e2e-dapp/add-wallet-support",
        "tutorials/build-e2e-dapp/fetch-data-from-chain",
        "tutorials/build-e2e-dapp/submit-data-to-chain",
        "tutorials/build-e2e-dapp/handle-tasks",
      ],
    },
    {
      type: "category",
      label: "Advanced builder guides",
      collapsible: true,
      collapsed: true,
      link: {
        type: "generated-index",
        title: "Advanced Builder Guides",
        description: "Take the next step into building complex applications on Aptos.",
        slug: "/category/advanced-builders",
      },
      items: [
        {
          type: "category",
          label: "Develop Locally",
          link: { type: "doc", id: "nodes/local-testnet/index" },
          collapsible: true,
          collapsed: true,
          items: [
            "guides/local-development-network",
            "nodes/local-testnet/run-a-local-testnet",
            "guides/running-a-local-multi-node-network",
          ],
        },
        "guides/transaction-management",
        {
          type: "category",
          label: "Learn about the Aptos Indexer",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "indexer/indexer-landing" },
          items: [
            {
              type: "category",
              label: "Indexer API",
              link: { type: "doc", id: "indexer/api/index" },
              collapsible: true,
              collapsed: true,
              items: ["indexer/api/labs-hosted", "indexer/api/self-hosted", "indexer/api/example-queries"],
            },
            {
              type: "category",
              label: "Custom Processors",
              link: { type: "doc", id: "indexer/custom-processors/index" },
              collapsible: true,
              collapsed: true,
              items: ["indexer/custom-processors/e2e-tutorial", "indexer/custom-processors/parsing-txns"],
            },
            {
              type: "category",
              label: "Transaction Stream Service",
              link: { type: "doc", id: "indexer/txn-stream/index" },
              collapsible: true,
              collapsed: true,
              items: [
                "indexer/txn-stream/labs-hosted",
                "indexer/txn-stream/self-hosted",
                "indexer/txn-stream/local-development",
              ],
            },
            {
              type: "category",
              label: "Legacy Indexer",
              link: { type: "doc", id: "indexer/legacy/index" },
              collapsible: true,
              collapsed: true,
              items: [
                "indexer/legacy/indexer-fullnode",
                "indexer/legacy/custom-data-model",
                "indexer/legacy/migration",
              ],
            },
          ],
        },
      ],
    },
  ],
  nodeSidebar: [
    {
      type: "html",
      value: "Run Nodes",
      className: "sidebar-title",
    },
    "nodes/nodes-landing",
    {
      type: "category",
      label: "Run a Validator",
      link: { type: "doc", id: "nodes/validator-node/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "nodes/validator-node/operator/node-requirements",
        {
          type: "category",
          label: "Running Validator Node",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "nodes/validator-node/operator/running-validator-node/index" },
          items: [
            "nodes/validator-node/operator/running-validator-node/using-aws",
            "nodes/validator-node/operator/running-validator-node/using-azure",
            "nodes/validator-node/operator/running-validator-node/using-gcp",
            "nodes/validator-node/operator/running-validator-node/using-docker",
            "nodes/validator-node/operator/running-validator-node/using-source-code",
            "nodes/validator-node/operator/update-validator-node",
          ],
        },
        "nodes/validator-node/operator/connect-to-aptos-network",
        "nodes/validator-node/operator/staking-pool-operations",
        "nodes/validator-node/operator/delegation-pool-operations",
        "nodes/validator-node/operator/shutting-down-nodes",
        {
          type: "category",
          label: "Voter",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "nodes/validator-node/voter/index" },
          items: [],
        },
        "nodes/leaderboard-metrics",
      ],
    },
    {
      type: "category",
      label: "Run a Fullnode",
      link: { type: "doc", id: "nodes/full-node/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "nodes/full-node/fullnode-source-code-or-docker",
        "nodes/full-node/bootstrap-fullnode",
        "nodes/full-node/aptos-db-restore",
        "nodes/full-node/update-fullnode-with-new-releases",
        "nodes/full-node/network-identity-fullnode",
        "nodes/full-node/fullnode-network-connections",
        "nodes/full-node/run-a-fullnode-on-gcp",
      ],
    },
    {
      type: "category",
      label: "Monitor Nodes",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "nodes/measure/index" },
      items: [
        "nodes/validator-node/operator/node-liveness-criteria",
        "nodes/measure/node-inspection-service",
        "nodes/measure/node-health-checker",
        "nodes/measure/node-health-checker-faq",
      ],
    },
    {
      type: "category",
      label: "Configure Nodes",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "nodes/identity-and-configuration" },
      items: [
        "reference/telemetry",
        "guides/state-sync",
        "guides/data-pruning",
        {
          type: "category",
          label: "Node Files",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "nodes/node-files-all-networks/node-files" },
          items: [
            "nodes/node-files-all-networks/node-files",
            "nodes/node-files-all-networks/node-files-testnet",
            "nodes/node-files-all-networks/node-files-devnet",
          ],
        },
      ],
    },
  ],
  refSidebar: [
    {
      type: "html",
      value: "Aptos References",
      className: "sidebar-title",
    },
    "nodes/aptos-api-spec",
    "reference/move",
    "reference/glossary",
  ],
  comSidebar: [
    {
      type: "html",
      value: "Aptos Community",
      className: "sidebar-title",
    },
    "community/index",
    "community/external-resources",
    "community/rust-coding-guidelines",
    "community/site-updates",
    "community/aptos-style",
    "community/contributors",
  ],
};

module.exports = sidebars;
