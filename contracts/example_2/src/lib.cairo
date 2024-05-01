use starknet::{ContractAddress, get_caller_address, contract_address_const};

#[starknet::interface]
pub trait IERC20<TContractState> {
    fn name(self: @TContractState) -> felt252;
    fn symbol(self: @TContractState) -> felt252;
    fn decimals(self: @TContractState) -> u8;
    fn totalSupply(self: @TContractState) -> u256;
    fn balanceOf(self: @TContractState, owner: ContractAddress) -> u256;

    fn transfer(ref self: TContractState, to: ContractAddress, value: u256) -> bool;
    fn transferFrom(ref self: TContractState, from: ContractAddress, to: ContractAddress, value: u256) -> bool;
    fn approve(ref self: TContractState, spender: ContractAddress, amount: u256) -> bool;
    fn allowance(self: @TContractState, owner: ContractAddress, spender: ContractAddress) -> u256;

    // As a test token, we want to allow anyone to mint
    fn mint(ref self: TContractState, account: ContractAddress, value: u256) -> bool;
}

#[starknet::contract]
mod ERC20 {
    use starknet::{ContractAddress, get_caller_address, contract_address_const, storage_access::StorageBaseAddress};
    
    #[storage]
    struct Storage {
        name: felt252,
        symbol: felt252,
        decimals: u8,
        totalSupply: u256, 
        balances: LegacyMap::<ContractAddress, u256>,
        allowances: LegacyMap::<(ContractAddress, ContractAddress), u256>,

        // balance: LegacyMap::<ContractAddress,u256>,
        // allowances: LegacyMap::<ContractAddress, ContractAddress, u256>
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        Transfer: Transfer,
        Approval: Approval,
    }

    #[derive(Drop, starknet::Event)]
    struct Transfer {
        #[key]
        from: ContractAddress,
        #[key]
        to: ContractAddress,
        value: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct Approval {
        #[key]
        owner: ContractAddress,
        #[key]
        spender: ContractAddress,
        value: u256,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState, 
        name: felt252,
        symbol: felt252,
        decimals: u8,
    ) {
       self.name.write(name);
       self.symbol.write(symbol);
       self.decimals.write(decimals);
    }

    #[abi(embed_v0)]
    impl ERC20Impl of super::IERC20<ContractState> {
        fn name(self: @ContractState) -> felt252 {
            self.name.read()
        }

        fn symbol(self: @ContractState) -> felt252 {
            self.symbol.read()
        }

        fn decimals(self: @ContractState) -> u8 {
            self.decimals.read()
        }

        fn totalSupply(self: @ContractState) -> u256 {
            self.totalSupply.read()
        }

        fn balanceOf(self: @ContractState, owner: ContractAddress) -> u256 {
            self.balances.read(owner)
        }

        fn allowance(self: @ContractState, owner: ContractAddress, spender: ContractAddress) -> u256 {
            self.allowances.read((owner, spender))
        }

        fn transfer(ref self: ContractState, to: ContractAddress, value: u256) -> bool {
            let msg_sender = get_caller_address();
            self._transfer(msg_sender, to, value);
            true
        }

        fn transferFrom(ref self: ContractState, from: ContractAddress, to: ContractAddress, value: u256) -> bool{
            let spender = get_caller_address();
            self._spendAllowance(from, spender, value);
            self._transfer(from, to, value);
            true
        }
        
        fn approve(ref self: ContractState, spender: ContractAddress, amount: u256) -> bool {
            let owner = get_caller_address();
            self._approve(owner, spender, amount);
            true
        }

        // for free minting of the token
        fn mint(ref self: ContractState, account: ContractAddress, value: u256) -> bool {
            self._mint(account, value);
            true
        }
    }



    // Internal functions
    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {

        fn _mint(
            ref self: ContractState,
            account: ContractAddress,
            value: u256
        ) {
            let addressZero: ContractAddress = contract_address_const::<0>();
            assert(account != addressZero, 'ERC20: Invalid Receiver');
            self._update(addressZero, account, value);
        }

        fn _burn(
            ref self: ContractState,
            account: ContractAddress,
            value: u256
        ) {
            let addressZero: ContractAddress = contract_address_const::<0>();
            assert(account != addressZero, 'ERC20: Invalid Sender');
            self._update(account, contract_address_const::<0>(), value);
        }

        fn _update(
            ref self: ContractState,
            from: ContractAddress,
            to: ContractAddress,
            value: u256
        ) {
            let addressZero = contract_address_const::<0>();
            if from == addressZero {
                self.totalSupply.write(self.totalSupply.read() + value);
            } else {
                let fromBalance = self.balances.read(from);
                assert(fromBalance > value, 'ERC20: Insufficient Balance');
                self.balances.write(from, fromBalance - value);
            }

            if to == addressZero {
                self.totalSupply.write(self.totalSupply.read() - value);
            } else {
                self.balances.write(to, self.balances.read(to) + value);
            }

            self.emit(Transfer {
                from: from,
                to: to,
                value: value
            })

        }

        // The transfer checks the address from and address to are not address zero then calls the internal update function
        fn _transfer(
            ref self: ContractState, 
            from: ContractAddress, 
            to: ContractAddress, 
            value: u256
        ) {
            let addressZero: ContractAddress = contract_address_const::<0>();
            assert(from != addressZero, 'ERC20: Address zero');           
            assert(to != addressZero, 'ERC20: Address zero');          
            self._update(from, to, value);
        }


        fn _approve(
            ref self: ContractState,
            owner: ContractAddress,
            spender: ContractAddress,
            value: u256
        ) {
            let addressZero: ContractAddress = contract_address_const::<0>();
            assert(owner != addressZero, 'ERC20: Invalid Approver');
            assert(spender != addressZero, 'ERC20: Invalid Spender');
            self.allowances.write((owner,spender), value);

            //TODO: Emit Approval event
        }

        fn _spendAllowance(
            ref self: ContractState,
            owner: ContractAddress,
            spender: ContractAddress,
            value: u256
        ) {
            assert(self.allowances.read((owner,spender)) > value, 'ERC20: Insufficient Allowance');
            self._approve(owner,spender, self.allowances.read((owner,spender)) - value);
        }
    }


}