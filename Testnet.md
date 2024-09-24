# Testnet deployment

Firstly check [installation](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective/#install-injectived/) and install all required packages and dependencies

Install scripts runner if haven't done yet

```sh
cargo install cargo-run-script
```

Build contract and optimize contract

```sh
cargo run-script build
cargo run-script optimize
```

<!-- Note: directory_to_which_you_cloned_cw-template must be an absolute path. The absolute path can easily be found by running the `pwd` command from inside the CosmWasm/cosmwasm directory.

```sh
docker run --name="injective-core-staging" \
-v=<directory_to_which_you_cloned_cw-template>/artifacts:/var/artifacts \
--entrypoint=sh public.ecr.aws/l9h3g6c6/injective-core:staging \
-c "tail -F anything"
```

```sh
docker run --name="injective-core-staging" \
-v=/mnt/c/Work/injex-finance/apps/contract/artifacts:/var/artifacts \
--entrypoint=sh public.ecr.aws/l9h3g6c6/injective-core:staging \
-c "tail -F anything"
```

Open a new terminal and go into the Docker container to initialize the chain:

```sh
docker exec -it injective-core-staging sh
```

```sh
# inside the "injective-core-staging" container
apk add jq
``` -->

if haven't done yet

Add test user

```sh
injectived keys add testuser
```

```sh
export INJ_ADDRESS= <your inj address>
```

Get some injective [faucet](https://bwarelabs.com/faucets/injective-testnet)

```sh
yes 12345678 | injectived tx wasm store artifacts/injex_vesting.wasm \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443
```

Query tx to get `code id`

```sh
injectived query tx BE4C2EC7C2E6841085884CFAFC91A9C3692E63CFF1715AC50DA867E4DCA6129A --node=https://testnet.sentry.tm.injective.network:443
```

Copy key: `code_id` and export to variables

```sh
export CODE_ID= <code_id of your stored contract>
```

Instantiate contract

```sh
INIT='{"instant_claim_percents":"1500","lock_minutes":"1","lock_periods":"5","injex_token":"inj","admin":"inj17lqukllrufjyzeg77xmvh6qped3w7wamxunhx0"}'
yes 12345678 | injectived tx wasm instantiate $CODE_ID $INIT \
--label="Injex Vesting" \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj \
--gas=2000000 \
--no-admin \
--node=https://testnet.sentry.tm.injective.network:443
```

Query tx to get contract address under `contract_address`

```sh
injectived query tx 97B1CC706296E926EBF8A50ACE7A48D31001F8352029C9F7AFA73FD1BAAF9DF3 --node=https://testnet.sentry.tm.injective.network:443
```

Get value `contract_address`

```sh
injectived tx wasm submit-proposal wasm-store /var/artifacts/injex_finance_injective.wasm \
--title "Upload Injex Finance Router and Limit Contract Wasm Code" \
--summary "## Upload Injex Finance Router and Limit Contract Wasm Code\\n\\n### Background\\n\\n Injex Protocol works by connecting to various DEXes within the Injective Network, pooling together liquidity, and ensuring users have access to the best possible rates with minimal slippage.\\n\\n### Summary\\n\\nThis proposal intends to upload the Injex Finance Router and Limit Contract for use by Injex Finance app on Injective.\\n\\nThese contracts are created for the Injex Finance dapp.\\n\\n### About Injex Finance\\n\\nInjex Finance, like pioneering liquidity aggregators, primarily offer aggregation; only this time primarily on the Injective network, through a groundbreaking aggregation protocol as is found across this document. The Injex aggregator strategically leverages the strengths of existing protocols while introducing innovative features specific to the Injective chain. By distributing trade weights across multiple liquidity pools concurrently, it aims to effectively reduce the impact of price slippage.\\n\\n### Contract Information\\n\\n- Compiler Version : cosmwasm/rust-optimizer:0.12.12\\n- Router checksum : 58703a9a91e31653b6c8e6afe82a550b008ca116ccc63462f0b50ace657b2c17  \\n- Limit contract checksum : 433250c91cad02896b987f0177d4aed282ec570ce15f44e771b7f461f8a2a82b  \\n\\n### TL;DR\\n\\n- This proposal is to upload the Injex Finance Router and Limit Contract on Injective\\n\\nBy voting YES on this proposal, you agree to uploading Injex Finance Router and Limit Contract as described in this proposal.\\n\\nBy voting NO on the proposal, you do not support uploading Injex Finance Router and Limit Contract as described in this proposal.\\n\\nBy voting NO WITH VETO, you find this proposal to be spam/irrelevant/malicious to governance, and contribute to burning 100 INJ deposit if NoWithVeto votes are greater than one third of the total voting power.\\n\\nBy voting ABSTAIN, you wish to contribute to quorum while formally declining to vote either for or against the proposal.\\n\\n### References \\n\\n- [Website]( https://injex.fi/ )\\n- [Twitter]( https://twitter.com/Injex_fi )\\n- [Medium]( https://medium.com/@injexfinance )\\n- [Whitepaper]( https://docs.injex.fi/ )\\n- [Telegram]( https://t.me/injexfi )" \
--from inj15arks2ynvufsumdmgkeaqgdxelsgjr3ggwn3dx \
--chain-id="injective-1" \
--node=https://sentry.tm.injective.network:443 \
--instantiate-everybody true \
--gas=50000000 \
--gas-prices=500000000inj \
--deposit=100000000000000000000inj
```

```sh
injectived tx xwasm batch-store-code-proposal \
--contract-files="router/artifacts/injex_aggregator.wasm,limit-orders/artifacts/injex_aggregator_limit.wasm" \
--batch-upload-proposal="batch-store-code-proposal.json" \
--from inj15arks2ynvufsumdmgkeaqgdxelsgjr3ggwn3dx \
--chain-id="injective-1" \
--node=https://sentry.tm.injective.network:443 \
--gas=50000000 \
--gas-prices=500000000inj \
--deposit=100000000000000000000inj
```
