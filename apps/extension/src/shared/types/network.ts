// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

export interface Network {
  faucetUrl?: string;
  name: string,
  nodeUrl: string;
}

export type Networks = Record<string, Network>;

export enum DefaultNetworks {
  Devnet = 'Devnet',
  Localhost = 'Localhost',
  Testnet = 'Testnet',
}

export const defaultCustomNetworks: Networks = {
  [DefaultNetworks.Localhost]: {
    faucetUrl: 'http://localhost:80',
    name: DefaultNetworks.Localhost,
    nodeUrl: 'http://localhost:8080',
  },
};

export const defaultNetworks: Networks = Object.freeze({
  [DefaultNetworks.Testnet]: {
    faucetUrl: 'https://faucet.testnet.aptoslabs.com',
    name: DefaultNetworks.Testnet,
    nodeUrl: 'https://testnet.aptoslabs.com',
  },
  [DefaultNetworks.Devnet]: {
    faucetUrl: 'https://faucet.devnet.aptoslabs.com',
    name: DefaultNetworks.Devnet,
    nodeUrl: 'https://fullnode.devnet.aptoslabs.com',
  },
} as const);

export const defaultNetworkName = DefaultNetworks.Testnet;
