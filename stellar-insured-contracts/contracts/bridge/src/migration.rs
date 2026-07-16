//! Bridge Contract Migration Implementation
//! 
//! Specific migration logic for the PropertyBridge contract using the
//! common migration framework.

use crate::storage::{DataKey, MAX_HISTORY_ITEMS};
use crate::types::{
    BridgeConfig, BridgeOperationStatus, BridgeTransaction, ChainBridgeInfo,
    MultisigBridgeRequest, PropertyMetadata, RecoveryAction,
};
use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec, vec};
use soroban_sdk::xdr::ToXdr;
use crate::migration_framework::{
    MigrationFramework, MigrationKey, MigrationOperation, MigrationStep, MigrationError,
    DefaultMigrationFramework,
};

/// Bridge-specific migration operations
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum BridgeMigrationOperation {
    /// Add new configuration fields
    AddConfigField(String),
    /// Migrate history storage format
    MigrateHistoryFormat,
    /// Update chain info structure
    UpdateChainInfo,
    /// Add new operation status
    AddOperationStatus,
}

/// Bridge migration manager
pub struct BridgeMigrationManager {
    framework: DefaultMigrationFramework,
}

impl BridgeMigrationManager {
    pub fn new() -> Self {
        Self {
            framework: DefaultMigrationFramework,
        }
    }

    /// Initialize bridge migration system
    pub fn initialize(&self, env: &Env) {
        self.framework.init_migration_system(env, 1);
    }

    /// Execute bridge-specific migration to version 2
    pub fn migrate_to_v2(&self, env: &Env) -> Result<u64, MigrationError> {
        let steps = vec![env,
            MigrationStep {
                step_id: 1,
                operation: MigrationOperation::AddField,
                description: String::from_str(&env, "Add emergency_pause field to BridgeConfig"),
                from_version: 1,
                to_version: 2,
                storage_key_pattern: String::from_str(&env, "Config"),
                is_critical: true,
            },
            MigrationStep {
                step_id: 2,
                operation: MigrationOperation::AddField,
                description: String::from_str(&env, "Add metadata_preservation field to BridgeConfig"),
                from_version: 1,
                to_version: 2,
                storage_key_pattern: String::from_str(&env, "Config"),
                is_critical: false,
            },
            MigrationStep {
                step_id: 3,
                operation: MigrationOperation::ModifyField,
                description: String::from_str(&env, "Update ChainBridgeInfo with supported_tokens field"),
                from_version: 1,
                to_version: 2,
                storage_key_pattern: String::from_str(&env, "ChainInfo(*)"),
                is_critical: true,
            },
        ];

        self.framework.begin_migration(env, 1, 2, steps)
    }

    /// Execute bridge-specific migration to version 3
    pub fn migrate_to_v3(&self, env: &Env) -> Result<u64, MigrationError> {
        let steps = vec![env,
            MigrationStep {
                step_id: 1,
                operation: MigrationOperation::AddField,
                description: String::from_str(&env, "Add gas_multiplier field to ChainBridgeInfo"),
                from_version: 2,
                to_version: 3,
                storage_key_pattern: String::from_str(&env, "ChainInfo(*)"),
                is_critical: false,
            },
            MigrationStep {
                step_id: 2,
                operation: MigrationOperation::AddField,
                description: String::from_str(&env, "Add confirmation_blocks field to ChainBridgeInfo"),
                from_version: 2,
                to_version: 3,
                storage_key_pattern: String::from_str(&env, "ChainInfo(*)"),
                is_critical: false,
            },
        ];

        self.framework.begin_migration(env, 2, 3, steps)
    }

    /// Execute specific migration step for bridge
    pub fn execute_bridge_step(
        &self,
        env: &Env,
        migration_id: u64,
        step_id: u32,
    ) -> Result<(), MigrationError> {
        match step_id {
            1 => self.migrate_config_v2(env),
            2 => self.migrate_metadata_v2(env),
            3 => self.migrate_chain_info_v2(env),
            4 => self.migrate_gas_multiplier_v3(env),
            5 => self.migrate_confirmation_blocks_v3(env),
            _ => Err(MigrationError::StepNotFound),
        }
    }

    /// Migrate BridgeConfig to v2 (add emergency_pause and metadata_preservation)
    fn migrate_config_v2(&self, env: &Env) -> Result<(), MigrationError> {
        let mut config: BridgeConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MigrationError::StorageCorruption)?;

        // Add new fields with default values
        config.emergency_pause = false;
        config.metadata_preservation = true;

        env.storage().instance().set(&DataKey::Config, &config);
        Ok(())
    }

    /// Migrate metadata preservation setting
    fn migrate_metadata_v2(&self, env: &Env) -> Result<(), MigrationError> {
        // This step would handle any metadata-specific migration logic
        // For now, it's a placeholder as the field was added in step 1
        Ok(())
    }

    /// Migrate ChainBridgeInfo to v2 (add supported_tokens)
    fn migrate_chain_info_v2(&self, env: &Env) -> Result<(), MigrationError> {
        // Get all chain IDs from supported chains in config
        let config: BridgeConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MigrationError::StorageCorruption)?;

        for chain_id in config.supported_chains.iter() {
            let mut chain_info: ChainBridgeInfo = env
                .storage()
                .persistent()
                .get(&DataKey::ChainInfo(chain_id))
                .ok_or(MigrationError::StorageCorruption)?;

            // Add supported_tokens field with empty vector
            chain_info.supported_tokens = Vec::new(&env);

            env.storage()
                .persistent()
                .set(&DataKey::ChainInfo(chain_id), &chain_info);
        }

        Ok(())
    }

    /// Migrate gas_multiplier field for v3
    fn migrate_gas_multiplier_v3(&self, env: &Env) -> Result<(), MigrationError> {
        let config: BridgeConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MigrationError::StorageCorruption)?;

        for chain_id in config.supported_chains.iter() {
            let mut chain_info: ChainBridgeInfo = env
                .storage()
                .persistent()
                .get(&DataKey::ChainInfo(chain_id))
                .ok_or(MigrationError::StorageCorruption)?;

            // Add gas_multiplier with default value
            chain_info.gas_multiplier = 100;

            env.storage()
                .persistent()
                .set(&DataKey::ChainInfo(chain_id), &chain_info);
        }

        Ok(())
    }

    /// Migrate confirmation_blocks field for v3
    fn migrate_confirmation_blocks_v3(&self, env: &Env) -> Result<(), MigrationError> {
        let config: BridgeConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MigrationError::StorageCorruption)?;

        for chain_id in config.supported_chains.iter() {
            let mut chain_info: ChainBridgeInfo = env
                .storage()
                .persistent()
                .get(&DataKey::ChainInfo(chain_id))
                .ok_or(MigrationError::StorageCorruption)?;

            // Add confirmation_blocks with default value
            chain_info.confirmation_blocks = 6;

            env.storage()
                .persistent()
                .set(&DataKey::ChainInfo(chain_id), &chain_info);
        }

        Ok(())
    }

    /// Validate bridge data integrity after migration
    pub fn validate_bridge_data(&self, env: &Env) -> Result<bool, MigrationError> {
        // Check critical data exists
        if !env.storage().instance().has(&DataKey::Config) {
            return Err(MigrationError::StorageCorruption);
        }

        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(MigrationError::StorageCorruption);
        }

        // Validate config structure
        let config: BridgeConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MigrationError::StorageCorruption)?;

        // Check required fields exist
        if config.supported_chains.is_empty() {
            return Err(MigrationError::StorageCorruption);
        }

        // Validate chain info for each supported chain
        for chain_id in config.supported_chains.iter() {
            let chain_info: ChainBridgeInfo = env
                .storage()
                .persistent()
                .get(&DataKey::ChainInfo(chain_id))
                .ok_or(MigrationError::StorageCorruption)?;

            if chain_info.chain_id != chain_id {
                return Err(MigrationError::StorageCorruption);
            }
        }

        Ok(true)
    }

    /// Create backup of critical bridge data
    pub fn create_backup(&self, env: &Env) -> Result<BytesN<32>, MigrationError> {
        // Backup critical data
        let config: BridgeConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MigrationError::StorageCorruption)?;

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(MigrationError::StorageCorruption)?;

        // Create checksum of backed up data
        let data = (config, admin);
        let checksum = env.crypto().sha256(&data.to_xdr(env));

        // Store backup checksum
        env.storage()
            .instance()
            .set(&MigrationKey::BackupChecksum(String::from_str(&env, "bridge_v1")), &checksum);

        Ok(checksum)
    }

    /// Restore bridge data from backup
    pub fn restore_from_backup(&self, env: &Env, checksum: &BytesN<32>) -> Result<(), MigrationError> {
        let stored_checksum: BytesN<32> = env
            .storage()
            .instance()
            .get(&MigrationKey::BackupChecksum(String::from_str(&env, "bridge_v1")))
            .ok_or(MigrationError::RollbackFailed)?;

        if stored_checksum != *checksum {
            return Err(MigrationError::RollbackFailed);
        }

        // Implementation would restore actual data from backup storage
        // For now, this is a placeholder
        Ok(())
    }
}

impl MigrationFramework for BridgeMigrationManager {
    fn init_migration_system(&self, env: &Env, initial_version: u32) {
        self.framework.init_migration_system(env, initial_version);
    }

    fn begin_migration(
        &self,
        env: &Env,
        from_version: u32,
        to_version: u32,
        steps: Vec<MigrationStep>,
    ) -> Result<u64, MigrationError> {
        self.framework.begin_migration(env, from_version, to_version, steps)
    }

    fn execute_step(
        &self,
        env: &Env,
        migration_id: u64,
        step_id: u32,
    ) -> Result<(), MigrationError> {
        self.execute_bridge_step(env, migration_id, step_id)
    }

    fn complete_migration(&self, env: &Env, migration_id: u64) -> Result<(), MigrationError> {
        // Validate data before completing
        self.validate_bridge_data(env)?;
        self.framework.complete_migration(env, migration_id)
    }

    fn rollback_migration(&self, env: &Env, migration_id: u64) -> Result<(), MigrationError> {
        self.framework.rollback_migration(env, migration_id)
    }

    fn get_version(&self, env: &Env) -> u32 {
        self.framework.get_version(env)
    }

    fn validate_migration(&self, env: &Env, steps: Vec<MigrationStep>) -> Result<(), MigrationError> {
        self.framework.validate_migration(env, steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token};

    struct TestContext<'a> {
        env: Env,
        admin: Address,
        operators: Vec<Address>,
        fee_token: token::Client<'a>,
        fee_admin_client: token::StellarAssetClient<'a>,
        fee_recipient: Address,
        bridge_client: crate::PropertyBridgeClient<'a>,
        user: Address,
    }

    fn setup_test<'a>() -> TestContext<'a> {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let operator2 = Address::generate(&env);
        let operator3 = Address::generate(&env);
        let fee_recipient = Address::generate(&env);
        let user = Address::generate(&env);

        let token_admin = Address::generate(&env);
        let token_address = env.register_stellar_asset_contract(token_admin.clone());
        let fee_token = token::Client::new(&env, &token_address);
        let fee_admin_client = token::StellarAssetClient::new(&env, &token_address);

        // Deploy bridge contract
        let bridge_id = env.register_contract(None, crate::PropertyBridge);
        let bridge_client = crate::PropertyBridgeClient::new(&env, &bridge_id);

        // Initialize bridge
        let mut supported_chains = Vec::new(&env);
        supported_chains.push_back(2); // Chain 2
        
        bridge_client.init(
            &admin,
            &supported_chains,
            &2, // min signatures
            &3, // max signatures
            &3600, // default timeout
            &100000, // gas limit
            &100, // service fee
            &token_address,
            &fee_recipient,
        );

        // Add extra operators (admin is already added as operator in init)
        bridge_client.add_operator(&admin, &operator2);
        bridge_client.add_operator(&admin, &operator3);

        let mut operators = Vec::new(&env);
        operators.push_back(admin.clone());
        operators.push_back(operator2.clone());
        operators.push_back(operator3.clone());

        // Mint tokens to user for service fee
        fee_admin_client.mint(&user, &1000);

        TestContext {
            env,
            admin,
            operators,
            fee_token,
            fee_admin_client,
            fee_recipient,
            bridge_client,
            user,
        }
    }

    #[test]
    fn test_rogue_operator_veto_success() {
        let ctx = setup_test();
        let metadata = PropertyMetadata {
            location: String::from_str(&ctx.env, "USA"),
            size: 1500,
            legal_description: String::from_str(&ctx.env, "Lot 4"),
            valuation: 500_000,
            documents_url: String::from_str(&ctx.env, "http://docs.com"),
        };

        // User initiates bridge request
        let request_id = ctx.bridge_client.initiate_bridge_multisig(
            &ctx.user,
            &1, // token_id
            &2, // destination_chain
            &ctx.user, // recipient
            &2, // required_signatures
            &Some(3600), // timeout_seconds
            &metadata,
            &1, // nonce
        );

        // Verify request created and status is Pending
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        assert_eq!(request.status, BridgeOperationStatus::Pending);

        // Operator 1 (admin) approves
        ctx.bridge_client.sign_bridge_request(&ctx.admin, &request_id, &true);
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        assert_eq!(request.status, BridgeOperationStatus::Pending);

        // Operator 2 (rogue/veto) rejects
        ctx.bridge_client.sign_bridge_request(&ctx.operators.get(1).unwrap(), &request_id, &false);
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        // It should still be Pending because threshold for failure (2 rejections) is not met!
        assert_eq!(request.status, BridgeOperationStatus::Pending);

        // Operator 3 approves (reaching required_signatures = 2 approvals)
        ctx.bridge_client.sign_bridge_request(&ctx.operators.get(2).unwrap(), &request_id, &true);
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        // It should now be Locked (quorum achieved)!
        assert_eq!(request.status, BridgeOperationStatus::Locked);
    }

    #[test]
    fn test_failed_by_rejections() {
        let ctx = setup_test();
        let metadata = PropertyMetadata {
            location: String::from_str(&ctx.env, "USA"),
            size: 1500,
            legal_description: String::from_str(&ctx.env, "Lot 4"),
            valuation: 500_000,
            documents_url: String::from_str(&ctx.env, "http://docs.com"),
        };

        // User initiates bridge request
        let request_id = ctx.bridge_client.initiate_bridge_multisig(
            &ctx.user,
            &1, // token_id
            &2, // destination_chain
            &ctx.user, // recipient
            &2, // required_signatures
            &Some(3600), // timeout_seconds
            &metadata,
            &1, // nonce
        );

        // Operator 1 rejects
        ctx.bridge_client.sign_bridge_request(&ctx.admin, &request_id, &false);
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        assert_eq!(request.status, BridgeOperationStatus::Pending);

        // Operator 2 rejects (rejections.len() = 2 >= required_signatures)
        ctx.bridge_client.sign_bridge_request(&ctx.operators.get(1).unwrap(), &request_id, &false);
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        // It should be Failed now
        assert_eq!(request.status, BridgeOperationStatus::Failed);
    }

    #[test]
    fn test_service_fee_escrow_and_refund() {
        let ctx = setup_test();
        let metadata = PropertyMetadata {
            location: String::from_str(&ctx.env, "USA"),
            size: 1500,
            legal_description: String::from_str(&ctx.env, "Lot 4"),
            valuation: 500_000,
            documents_url: String::from_str(&ctx.env, "http://docs.com"),
        };

        let initial_user_balance = ctx.fee_token.balance(&ctx.user);

        // User initiates bridge request (charges 100 service fee)
        let request_id = ctx.bridge_client.initiate_bridge_multisig(
            &ctx.user,
            &1, // token_id
            &2, // destination_chain
            &ctx.user, // recipient
            &2, // required_signatures
            &Some(3600), // timeout_seconds
            &metadata,
            &1, // nonce
        );

        // Check fee escrowed
        assert_eq!(ctx.fee_token.balance(&ctx.user), initial_user_balance - 100);
        assert_eq!(ctx.fee_token.balance(&ctx.bridge_client.address), 100);

        // Case A: Fails by rejections, then recovered with CancelBridge -> refunds fee
        ctx.bridge_client.sign_bridge_request(&ctx.admin, &request_id, &false);
        ctx.bridge_client.sign_bridge_request(&ctx.operators.get(1).unwrap(), &request_id, &false);
        
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        assert_eq!(request.status, BridgeOperationStatus::Failed);

        // Admin recovers the failed bridge
        ctx.bridge_client.recover_failed_bridge(&ctx.admin, &request_id, &RecoveryAction::CancelBridge);

        // User balance should be restored to initial
        assert_eq!(ctx.fee_token.balance(&ctx.user), initial_user_balance);
        assert_eq!(ctx.fee_token.balance(&ctx.bridge_client.address), 0);
    }

    #[test]
    fn test_service_fee_refund_on_retry() {
        let ctx = setup_test();
        let metadata = PropertyMetadata {
            location: String::from_str(&ctx.env, "USA"),
            size: 1500,
            legal_description: String::from_str(&ctx.env, "Lot 4"),
            valuation: 500_000,
            documents_url: String::from_str(&ctx.env, "http://docs.com"),
        };

        let initial_user_balance = ctx.fee_token.balance(&ctx.user);

        // User initiates bridge request (charges 100 service fee)
        let request_id = ctx.bridge_client.initiate_bridge_multisig(
            &ctx.user,
            &1, // token_id
            &2, // destination_chain
            &ctx.user, // recipient
            &2, // required_signatures
            &Some(3600), // timeout_seconds
            &metadata,
            &1, // nonce
        );

        // Operator 1 and 2 reject it
        ctx.bridge_client.sign_bridge_request(&ctx.admin, &request_id, &false);
        ctx.bridge_client.sign_bridge_request(&ctx.operators.get(1).unwrap(), &request_id, &false);

        // Admin recovers it with RetryBridge -> refunds fee
        ctx.bridge_client.recover_failed_bridge(&ctx.admin, &request_id, &RecoveryAction::RetryBridge);

        // Fee should be refunded
        assert_eq!(ctx.fee_token.balance(&ctx.user), initial_user_balance);
        
        // Request status should be Pending and signatures/rejections cleared
        let request = ctx.bridge_client.get_request(&request_id).unwrap();
        assert_eq!(request.status, BridgeOperationStatus::Pending);
        assert_eq!(request.signatures.len(), 0);
        assert_eq!(request.rejections.len(), 0);
    }

    #[test]
    fn test_service_fee_released_on_execution() {
        let ctx = setup_test();
        let metadata = PropertyMetadata {
            location: String::from_str(&ctx.env, "USA"),
            size: 1500,
            legal_description: String::from_str(&ctx.env, "Lot 4"),
            valuation: 500_000,
            documents_url: String::from_str(&ctx.env, "http://docs.com"),
        };

        let initial_user_balance = ctx.fee_token.balance(&ctx.user);

        // User initiates bridge request
        let request_id = ctx.bridge_client.initiate_bridge_multisig(
            &ctx.user,
            &1, // token_id
            &2, // destination_chain
            &ctx.user, // recipient
            &2, // required_signatures
            &Some(3600), // timeout_seconds
            &metadata,
            &1, // nonce
        );

        // Operators approve it
        ctx.bridge_client.sign_bridge_request(&ctx.admin, &request_id, &true);
        ctx.bridge_client.sign_bridge_request(&ctx.operators.get(1).unwrap(), &request_id, &true);

        // Execute bridge -> transfers escrowed fee to fee_recipient
        ctx.bridge_client.execute_bridge(&ctx.admin, &request_id);

        // Check balances
        assert_eq!(ctx.fee_token.balance(&ctx.user), initial_user_balance - 100);
        assert_eq!(ctx.fee_token.balance(&ctx.fee_recipient), 100);
        assert_eq!(ctx.fee_token.balance(&ctx.bridge_client.address), 0);
    }
}
