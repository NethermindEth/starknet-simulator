use starknet::{ContractAddress, get_caller_address, contract_address_const};
use zeroable::Zeroable;

#[starknet::interface]
pub trait IERC20<TContractState> {
    fn name(ref self: TContractState) -> felt252;
    fn symbol(self: @TContractState) -> felt252;
    fn decimals(self: @TContractState) -> u8;
    fn totalSupply(self: @TContractState) -> u252;
    fn balanceOf(self: @TContractState, owner: ContractAddress) -> u252;

    fn transfer(ref self: TContractState, to: ContractAddress, value: u256) -> boolean;
    fn transferFrom(ref self: TContractState, from: ContractAddress, to: ContractAddress, value: u256) -> u256;
    fn approve(ref self: TContractState, spender: ContractAddress, amount: ContractAddress) -> boolean;
    fn allowance(self: @TContractState, owner: ContractAddress, spender: ContractAddress) -> u256;

    // As a test token, we want to allow anyone to mint
    fn mint(ref self: TContractState, account: ContractAddress, value: u256) -> boolean;
}

#[starknet::contract]
mod ERC20 {
    #[storage]
    struct Storage {
        name: felt252,
        symbol: felt252,
        decimals: u8,
        totalSupply: u256, 
        balance: LegacyMap::<ContractAddress,u256>,
        allowances: LegacyMap::<(ContractAddress, ContractAddress), u256>
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
            self.balance.read(owner)
        }

        fn allowance(self: @ContractState, owner: ContractAddress, spender: ContractAddress) -> u256 {
            self.allowances.read(owner, spender)
        }

        fn transfer(ref self: TContractState, to: ContractAddress, value: u256) -> boolean {
            let msg_sender = get_caller_address();
            self._transfer(msg_sender, to, value);
            true
        }

        fn transferFrom(ref self: TContractState, from: ContractAddress, to: ContractAddress, value: u256) -> u256{
            let spender = get_caller_address();
            self._spendAllowance(from, spender, value);
            self._transfer(from, to, value);
            true
        };
        
        fn approve(ref self: TContractState, spender: ContractAddress, amount: ContractAddress) -> boolean {
            let owner = get_caller_address();
            self._approve(owner, spender, amount);
            true
        };

        fn allowance(ref self: TContractState, owner: ContractAddress, spender: ContractAddress) -> u256 {
            self.allowances.read(owner, spender)
        };

        // for free minting of the token
        fn mint(ref self: TContractState, account: ContractAddress, value: ContractAddress) -> boolean {
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
            assert(!account.is_zero(), "ERC20: Invalid Receiver");
            self._update(contract_address_const::<0>(), account, value);
        }

        fn _burn(
            ref self: ContractState,
            account: ContractAddress,
            value: ContractAddress
        ) {
            assert(!account.is_zero(), "ERC20: Invalid Sender");
            self._update(account, contract_address_const::<0>(), value);
        }

        fn _update(
            ref self: ContractState,
            from: ContractAddress,
            to: ContractAddress,
            value: u256
        ) {
            if from.is_zero() {
                self.totalSupply.write(self.totalSupply.read() + value);
            } else {
                let fromBalance = self.balance.read(from);
                assert(fromBalance > value, "ERC20: Insufficient from Balance");
                self.balance.write(from, fromBalance - value);
            }

            if to.is_zero() {
                self.totalSupply.write(self.totalSupply.read() - value);
            } else {
                self.balance.write(to, self.balance.read(to) + value);
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
            assert(!from.is_zero(), "ERC20: Address zero");           
            assert(!to.is_zero(), "ERC20: Address zero");          
            self._update(from, to, value);
        }


        fn _approve(
            ref self: ContractAddress,
            owner: ContractAddress,
            spender: ContractAddress,
            value: u256
        ) {
            assert(!owner.is_zero(), "ERC20: Invalid Approver");
            assert(!spender.is_zero(), "ERC20: Invalid Spender");
            self.allowances.write(owner,spender,value);

            //TODO: Emit Approval event
        }

        fn _spendAllowance(
            ref self: ContractState,
            owner: ContractAddress,
            spender: ContractAddress,
            value: ContractAddress
        ) {
            assert(self.allowance(owner,spender) > value, "ERC20: Insufficient Allowance");
            self._approve(owner,spender, self.allowance.read(owner) - value);
        }
    }


}