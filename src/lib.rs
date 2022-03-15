#![no_std] // No standard Lib imported

elrond_wasm::imports!(); // import the depandencies in cargo.toml
elrond_wasm::derive_imports!(); // for #[derive()]

// define, the trait is a contract
#[elrond_wasm::contract]
pub trait Crowdfunding {

    //Storage
    #[view(getTarget)]  // view fn enable to fetch the map through api
    #[storage_mapper("target")] // defining a map to store
    fn target(&self) -> SingleValueMapper<BigUint>; // mapping the function to a Big-Unsigned-Integer

    #[view(getDeadline)]
    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<u64>;

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, donor: &ManagedAddress) -> SingleValueMapper<BigUint>;


    //Constructor
    #[init]
    fn init(&self, target: BigUint, deadline: u64) {
        self.target().set(&target); // mapping the current "target" instance with the "target:Biguint" given
        self.deadline().set(&deadline);
    }

    #[endpoint] //all payable fn needs to be endpoint but not vise-a-versa
    #[payable("*")] // for payable fn
    fn fund(
        &self,
        #[payment_amount] payment: BigUint // defining the curency data-type
    ) -> SCResult<()>{

        //cheacking if the funding deadline has exceeded or not
        let current_time = self.blockchain().get_block_nonce(); //gets the correct block nounce, which is a refference of time
        require!(current_time > self.deadline().get(), "cannot fund after deadline");


        /// updating/mapping the funder to the payed amount
        let caller = self.blockchain().get_caller(); // similar to msg.sender/get_the_caller of this function

        //get the 'deposit' mapp by the key 'caller'
        //thn update the value by payment
        self.deposit(&caller).update(|deposit| *deposit += payment);

        Ok(())
    }


    // status fn to return the current status of the program
    // if (current_time < deadline) -> FundingPeriod
    // else if (total_funds >= target) -> Succesful
    // else -> Failed
    #[view]
    fn status(&self) -> Status {
        if self.blockchain().get_block_nonce() <= self.deadline().get() {
            Status::FundingPeriod
        } else if self.get_current_funds() >= self.target().get() {
            Status::Successful
        }else {
            Status::Failed
        }
    }

    //fn to get the total funded amount
    #[view(getCurrentFunds)]
    fn get_current_funds(&self) -> BigUint {
        self.blockchain().get_sc_balance(&TokenIdentifier::egld(), 0) // to get the smartcontract_balance
    }

    //fn for claming the funds
    #[endpoint]
    fn claim(&self) -> SCResult<()> {
        // match is similar to switch
        match self.status() {
            // when status = FundingPeriod
            Status::FundingPeriod => sc_error!("cannot claim before deadline"), //smartcontract error
            // when status = Successful
            Status::Successful => {
                let caller = self.blockchain().get_caller(); // similar to msg.sender
                require!(
                    caller == self.blockchain().get_owner_address(), // gets the address of the owner
                    "only owner can claim successful funding"
                );

                let sc_balance = self.get_current_funds();
                self.send()
                    .direct(&caller, &TokenIdentifier::egld(),0, &sc_balance, b"claim"); // send the total fund stored in smartcontract to the "fn_caller", who is the owner of this smartcontract

                Ok(())
            },

            //when status = Failed
            Status::Failed => {
                let caller = self.blockchain().get_caller();
                let deposit = self.deposit(&caller).get(); // get the amount deposit by this caller from the storage map "deposit"

                if deposit > 0 {
                    self.deposit(&caller).clear(); //clear the amount value associated with this caller in the map
                    self.send()
                        .direct(&caller, &TokenIdentifier::egld(),0, &deposit, b"claim") // transfer the deposited amount back to the caller
                }

                Ok(())
            },
        }
    }

}

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone, Copy)]
// The #[derive] keyword in Rust allows you to automatically implement certain traits for your type. 
// TopEncode and TopDecode mean that objects of this type are serializable, which means they can be interpreted from/to a string of bytes.

// TypeAbi is needed to export the type when you want to interact with the already deployed contract.

// PartialEq Rust traits that allow your type instances to be compared with the == operator

// the Clone and Copy traits allow your object instances to be clone/copied respectively.
pub enum Status {
    FundingPeriod,
    Successful,
    Failed
}