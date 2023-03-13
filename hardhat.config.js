/** @type import('hardhat/config').HardhatUserConfig */
require("@nomicfoundation/hardhat-foundry");
require("dotenv").config();
require("@nomiclabs/hardhat-waffle");

module.exports = {
  solidity: "0.8.18",
  networks: {
    goerli: {
      url: process.env.GOERLI_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
  },
};
