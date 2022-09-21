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
    "whats-new-in-docs",
    "guides/getting-started",
    "aptos-developer-resources",
    {
      type: "category",
      label: "Developer Tutorials",
      link: { type: "doc", id: "tutorials/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "tutorials/first-transaction",
        "tutorials/your-first-nft",
        "tutorials/first-move-module",
        "tutorials/first-dapp",
        "tutorials/first-coin",
      ],
    },
    {
      type: "category",
      label: "Concepts",
      link: { type: "doc", id: "concepts/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "concepts/basics-txns-states",
        "concepts/basics-accounts",
        "concepts/basics-events",
        "concepts/basics-gas-txn-fee",
        {
          type: "category",
          label: "Coin and Token",
          link: { type: "doc", id: "concepts/coin-and-token/index" },
          collapsible: true,
          collapsed: true,
          items: ["concepts/coin-and-token/aptos-coin", "concepts/coin-and-token/aptos-token"],
        },
        "concepts/basics-merkle-proof",
        "concepts/basics-fullnodes",
        "concepts/basics-validator-nodes",
        "concepts/basics-node-networks-sync",
        "concepts/state-sync",
        "concepts/staking",
        "concepts/governance",
      ],
    },
    {
      type: "category",
      label: "Guides",
      link: { type: "doc", id: "guides/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "guides/basics-life-of-txn",
        "guides/sign-a-transaction",
        "guides/interacting-with-the-blockchain",
        "guides/building-your-own-wallet",
        "guides/install-petra-wallet",
        "guides/building-wallet-extension",
        "guides/system-integrators-guide",
        "guides/local-testnet-dev-flow",
        "guides/running-a-local-multi-node-network",
        {
          type: "category",
          label: "Move Guides",
          link: { type: "doc", id: "guides/move-guides/index" },
          collapsible: true,
          collapsed: true,
          items: ["guides/move-guides/move-on-aptos", "guides/move-guides/upgrading-move-code"],
        },
      ],
    },
    {
      type: "category",
      label: "Nodes",
      link: { type: "doc", id: "nodes/index" },
      collapsible: true,
      collapsed: true,
      items: [
        "nodes/aptos-deployments",
        {
          type: "category",
          label: "AIT-3",
          link: { type: "doc", id: "nodes/ait/index" },
          collapsible: true,
          collapsed: true,
          items: [
            "nodes/ait/whats-new-in-ait3",
            "nodes/ait/steps-in-ait3",
            "nodes/ait/node-requirements",
            "nodes/ait/node-liveness-criteria",
            "nodes/ait/ait3-leaderboard-metrics",
            "nodes/ait/connect-to-testnet",
            "nodes/ait/additional-doc",
          ],
        },
        {
          type: "category",
          label: "Validators",
          link: { type: "doc", id: "nodes/validator-node/index" },
          collapsible: true,
          collapsed: true,
          items: [
            "nodes/validator-node/using-aws",
            "nodes/validator-node/using-azure",
            "nodes/validator-node/using-gcp",
            "nodes/validator-node/using-docker",
            "nodes/validator-node/using-source-code",
          ],
        },
        {
          type: "category",
          label: "Public Fullnode",
          link: { type: "doc", id: "nodes/full-node/index" },
          collapsible: true,
          collapsed: true,
          items: [
            "nodes/full-node/fullnode-source-code-or-docker",
            "nodes/full-node/update-fullnode-with-new-releases",
            "nodes/full-node/network-identity-fullnode",
            "nodes/full-node/troubleshooting-fullnode",
            "nodes/full-node/run-a-fullnode-on-gcp",
            "nodes/full-node/bootstrap-fullnode",
          ],
        },
        {
          type: "category",
          label: "Local Testnet",
          link: { type: "doc", id: "nodes/local-testnet/index" },
          collapsible: true,
          collapsed: true,
          items: ["nodes/local-testnet/using-cli-to-run-a-local-testnet", "nodes/local-testnet/run-a-local-testnet"],
        },
        {
          type: "category",
          label: "Node Health Checker",
          link: { type: "doc", id: "nodes/node-health-checker/index" },
          collapsible: true,
          collapsed: true,
          items: ["nodes/node-health-checker/node-health-checker-faq"],
        },
      ],
    },
    {
      type: "category",
      label: "SDKs",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "sdks/index" },
      items: [
        "sdks/python-sdk",
        {
          type: "category",
          label: "Typescript SDK",
          link: { type: "doc", id: "sdks/ts-sdk/index" },
          collapsible: true,
          collapsed: true,
          items: ["sdks/ts-sdk/typescript-sdk", "sdks/ts-sdk/typescript-sdk-overview"],
        },
        "sdks/rust-sdk",
      ],
    },
    {
      type: "category",
      label: "Aptos CLI",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "cli-tools/aptos-cli-tool/index" },
      items: ["cli-tools/aptos-cli-tool/install-aptos-cli", "cli-tools/aptos-cli-tool/use-aptos-cli"],
    },
    "reference/telemetry",
    {
      type: "category",
      label: "Aptos White Paper",
      collapsible: true,
      collapsed: true,
      link: { type: "doc", id: "aptos-white-paper/index" },
      items: ["aptos-white-paper/in-korean"],
    },
    "reference/glossary",
  ],
};

module.exports = sidebars;
