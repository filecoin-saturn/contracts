const hre = require("hardhat");

async function main() {
  // hardhat-ethers
  const Contract = await hre.ethers.getContractFactory("Evaluator");
  const contract = await Contract.deploy(
    "0xf4728721157A58b0509c8c109Ec2AF726B562D6A" // CHANGE WITH THE DESIRED PUBLIC ADDRESS !!!
  );

  await contract.deployed();
  console.log(`Deployed to ${contract.address}`);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
