mod erc20;

#[starknet::contract]
mod StakeSystem {
    use super::erc20;

    #[storage]
    struct Storage {
    }

    #[constructor]
    fn constructor(
        ref self: ContractState
    ) {}
    
}