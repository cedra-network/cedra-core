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
    "index",
    {
      type: "html",
      value: "Get Started",
      className: "sidebar-title",
    },
    "whats-new-in-docs",
    {
      type: "category",
      label: "Latest Releases",
      collapsible: true,
      collapsed: false,
      link: { type: "doc", id: "releases/index" },
      items: ["releases/mainnet-release", "releases/testnet-release", "releases/devnet-release"],
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
      label: "Learn Aptos Concepts",
      link: { type: "doc", id: "concepts/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "concepts/accounts",
        "concepts/resources",
        "concepts/events",
        "concepts/txns-states",
        "concepts/gas-txn-fee",
        "concepts/blocks",
        "guides/basics-life-of-txn",
        "concepts/staking",
        "concepts/governance",
      ],
    },
    {
      type: "category",
      label: "Prepare Your Environment",
      link: { type: "doc", id: "guides/getting-started" },
      collapsible: true,
      collapsed: true,
      items: [
        "guides/use-aptos-explorer",
        "guides/install-petra-wallet",
        {
          type: "category",
          label: "Install the Aptos CLI",
          link: { type: "doc", id: "cli-tools/aptos-cli-tool/install-cli" },
          collapsible: true,
          collapsed: true,
          items: [
            "cli-tools/aptos-cli-tool/automated-install-aptos-cli",
            "cli-tools/build-aptos-cli-brew",
            "cli-tools/aptos-cli-tool/install-aptos-cli",
            "cli-tools/build-aptos-cli",
            "cli-tools/install-move-prover",
          ],
        },
        "cli-tools/aptos-cli-tool/use-aptos-cli",
        "guides/get-test-funds",
      ],
    },
    "guides/system-integrators-guide",
  ],
  appSidebar: [
    {
      type: "html",
      value: "Build Apps",
      className: "sidebar-title",
    },
    {
      type: "category",
      label: "Follow the Aptos Standards",
      link: { type: "doc", id: "concepts/coin-and-token/index" },
      collapsible: true,
      collapsed: true,
      items: ["concepts/coin-and-token/aptos-coin", "concepts/coin-and-token/aptos-token", "guides/wallet-standard"],
    },
    {
      type: "category",
      label: "Read Blockchain Data",
      collapsible: true,
      collapsed: true,
      link: {
        type: "generated-index",
        title: "Read Blockchain Data",
        description: "Use the Aptos API and indexer to read the Aptos blockchain state.",
        slug: "/category/data",
        keywords: ["data"],
      },
      items: ["guides/aptos-apis", "guides/indexing"],
    },
    {
      type: "category",
      label: "Interact with the Blockchain",
      link: { type: "doc", id: "guides/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "guides/sign-a-transaction",
        "guides/resource-accounts",
        "guides/aptos-names-service-package",
        "guides/handle-aptos-errors",
      ],
    },
    {
      type: "category",
      label: "Develop with the SDKs",
      link: { type: "doc", id: "tutorials/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "tutorials/first-transaction",
        "tutorials/first-dapp",
        "tutorials/first-coin",
        "tutorials/first-multisig",
        "concepts/coin-and-token/propertymap-offchain",
      ],
    },
    {
      type: "category",
      label: "Integrate with Wallets",
      link: { type: "doc", id: "concepts/wallet-adapter-concept" },
      collapsible: true,
      collapsed: true,
      items: ["guides/wallet-adapter-for-dapp", "guides/wallet-adapter-for-wallets"],
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
  ],
  moveSidebar: [
    {
      type: "html",
      value: "Learn the Move Language",
      className: "sidebar-title",
    },
    "guides/move-guides/index",
    "guides/move-guides/move-on-aptos",
    "guides/move-guides/move-structure",
    "guides/move-guides/bytecode-dependencies",
    "concepts/base-gas",
    "guides/interacting-with-the-blockchain",
    "tutorials/first-move-module",
    {
      type: "category",
      label: "Aptos Move Book",
      link: { type: "doc", id: "guides/move-guides/book/SUMMARY" },
      collapsible: true,
      collapsed: true,
      items: [
        "guides/move-guides/book/introduction",
        "guides/move-guides/book/modules-and-scripts",
        "guides/move-guides/book/creating-coins",
        "guides/move-guides/book/integers",
        "guides/move-guides/book/bool",
        "guides/move-guides/book/address",
        "guides/move-guides/book/vector",
        "guides/move-guides/book/signer",
        "guides/move-guides/book/references",
        "guides/move-guides/book/tuples",
        "guides/move-guides/book/variables",
        "guides/move-guides/book/equality",
        "guides/move-guides/book/abort-and-assert",
        "guides/move-guides/book/conditionals",
        "guides/move-guides/book/loops",
        "guides/move-guides/book/functions",
        "guides/move-guides/book/structs-and-resources",
        "guides/move-guides/book/constants",
        "guides/move-guides/book/generics",
        "guides/move-guides/book/abilities",
        "guides/move-guides/book/uses",
        "guides/move-guides/book/friends",
        "guides/move-guides/book/packages",
        "guides/move-guides/book/package-upgrades",
        "guides/move-guides/book/unit-testing",
        "guides/move-guides/book/global-storage-structure",
        "guides/move-guides/book/global-storage-operators",
        "guides/move-guides/book/standard-library",
        "guides/move-guides/book/coding-conventions",
      ],
    },
  ],
  nftSidebar: [
    {
      type: "html",
      value: "Create Tokens",
      className: "sidebar-title",
    },
    {
      type: "category",
      label: "Create Tokens on Aptos",
      collapsible: false,
      collapsed: false,
      link: {
        type: "generated-index",
        title: "Create Tokens on Aptos",
        description: "Learn the various ways to mint and exchange tokens.",
        slug: "/category/nft",
        keywords: ["nft"],
      },
      items: [
        "concepts/coin-and-token/aptos-token-comparison",
        "tutorials/your-first-nft",
        "guides/move-guides/mint-nft-cli",
        "concepts/coin-and-token/onchain-data",
        "concepts/coin-and-token/nft-minting-tool",
        "concepts/coin-and-token/airdrop-aptos-tokens",
      ],
    },
  ],
  nodeSidebar: [
    {
      type: "html",
      value: "Run Nodes",
      className: "sidebar-title",
    },
    {
      type: "category",
      label: "Learn about Nodes",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "nodes/nodes-landing" },
      items: ["concepts/node-networks-sync", "nodes/aptos-deployments", "nodes/leaderboard-metrics"],
    },
    /** Delete during clean up
    {
      type: "category",
      label: "AIT-3",
      link: { type: "doc", id: "nodes/ait/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "nodes/ait/whats-new-in-ait3",
        "nodes/ait/steps-in-ait3",

      ],
    },  */
    {
      type: "category",
      label: "Develop Locally",
      link: { type: "doc", id: "nodes/local-testnet/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "guides/local-testnet-dev-flow",
        "nodes/local-testnet/run-a-local-testnet",
        "nodes/local-testnet/using-cli-to-run-a-local-testnet",
        "guides/running-a-local-multi-node-network",
      ],
    },
    {
      type: "category",
      label: "Run a Validator",
      link: { type: "doc", id: "nodes/validator-node/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "concepts/validator-nodes",
        {
          type: "category",
          label: "Owner",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "nodes/validator-node/owner/index" },
          items: [],
        },
        {
          type: "category",
          label: "Operator",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "nodes/validator-node/operator/index" },
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
          ],
        },
        {
          type: "category",
          label: "Voter",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "nodes/validator-node/voter/index" },
          items: [],
        },
      ],
    },
    {
      type: "category",
      label: "Run a Fullnode",
      link: { type: "doc", id: "nodes/full-node/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "concepts/fullnodes",
        "nodes/full-node/fullnode-source-code-or-docker",
        "nodes/full-node/bootstrap-fullnode",
        "nodes/full-node/update-fullnode-with-new-releases",
        "nodes/full-node/network-identity-fullnode",
        "nodes/full-node/fullnode-network-connections",
        "nodes/full-node/run-a-fullnode-on-gcp",
      ],
    },
    "nodes/indexer-fullnode",
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
          label: "Node Files For Mainnet",
          collapsible: true,
          collapsed: true,
          link: { type: "doc", id: "nodes/node-files-all-networks/node-files" },
          items: [
            "nodes/node-files-all-networks/node-files-devnet",
            "nodes/node-files-all-networks/node-files-testnet",
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
    "reference/index",
    "nodes/aptos-api-spec",
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
          items: ["sdks/ts-sdk/typescript-sdk-overview"],
        },
        "sdks/python-sdk",
        "sdks/rust-sdk",
        "sdks/unity-sdk",
      ],
    },
    "reference/move",
    "reference/glossary",
    "issues-and-workarounds",
  ],
  comSidebar: [
    {
      type: "html",
      value: "Aptos Community",
      className: "sidebar-title",
    },
    "community/index",
    {
      type: "category",
      label: "Community Highlights",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "community/contributions/index" },
      items: [
        "community/contributions/remix-ide-plugin",
      ],
    },
    "community/external-resources",
    "community/rust-coding-guidelines",
    "community/site-updates",
    "community/aptos-style",
  ],
};

module.exports = sidebars;
