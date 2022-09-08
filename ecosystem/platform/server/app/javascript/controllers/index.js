// This file is auto-generated by ./bin/rails stimulus:manifest:update
// Run that command whenever you add a new controller or create them with
// ./bin/rails generate stimulus controllerName

import { application } from "./application";

import ClaimNftController from "./claim_nft_controller";
application.register("claim-nft", ClaimNftController);

import ConnectWalletController from "./connect_wallet_controller";
application.register("connect-wallet", ConnectWalletController);

import DialogController from "./dialog_controller";
application.register("dialog", DialogController);

import HeaderController from "./header_controller";
application.register("header", HeaderController);

import MintedNftController from "./minted_nft_controller";
application.register("minted-nft", MintedNftController);

import ProjectImagesController from "./project_images_controller";
application.register("project-images", ProjectImagesController);

import RecaptchaController from "./recaptcha_controller";
application.register("recaptcha", RecaptchaController);

import RefreshController from "./refresh_controller";
application.register("refresh", RefreshController);

import ShakeController from "./shake_controller";
application.register("shake", ShakeController);

import VisibilityController from "./visibility_controller";
application.register("visibility", VisibilityController);

import TableRowController from "./table_row_controller";
application.register("table_row", TableRowController);
