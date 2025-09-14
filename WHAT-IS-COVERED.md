# Phase 1 Validation Errors and Warnings

DISCLAIMER: Not all of these errors exactly match those in cardano-ledger; most cover similar logic, but the "location" may differ. Additionally, warnings are not part of cardano-ledger validation—they are implemented here to highlight situations where a transaction might execute differently than you expect or it has some unnecessary things.

## Not Yet Covered

- Pre-Conway transaction validation
- Governance action proposal validation
- Voting for governance actions validation
- Byron-era address signature validation

## 1. AuxiliaryDataValidator (`auxiliary_data.rs`)

Validates auxiliary data and its hash consistency.

### Errors (3)
- **Auxiliary data hash mismatch** - The hash of the auxiliary data doesn't match the expected hash in the transaction body
- **Auxiliary data hash missing** - Transaction contains auxiliary data but the hash is missing from the transaction body
- **Auxiliary data hash present but not expected** - Transaction body contains auxiliary data hash but no auxiliary data is provided


## 2. BalanceValidator (`balance.rs`)

Validates transaction balance, deposits, refunds, and withdrawals.

### Errors (11)
- **Value not conserved** - The sum of inputs doesn't equal the sum of outputs (balance equation fails)
- **Treasury value mismatch** - The declared treasury value doesn't match the actual treasury value
- **Wrong requested withdrawal amount** - The withdrawal amount doesn't match the available reward balance
- **Withdrawal not allowed because not delegated to DRep** - Attempting withdrawal from stake credential not delegated to a DRep
- **Reward account not existing** - Attempting withdrawal from a non-existent reward account
- **Stake registration wrong deposit** - The deposit amount for stake registration doesn't match protocol parameters
- **DRep incorrect deposit** - The deposit amount for DRep registration doesn't match protocol parameters
- **Pool registration wrong deposit** - The deposit amount for pool registration doesn't match protocol parameters
- **Voting proposal incorrect deposit** - The deposit amount for governance proposal doesn't match protocol parameters
- **Stake deregistration wrong refund** - The refund amount for stake deregistration doesn't match the original deposit
- **DRep deregistration wrong refund** - The refund amount for DRep deregistration doesn't match the original deposit

### Warnings (2)
- **Cannot check stake deregistration refund** - Unable to verify the refund amount due to missing context information
- **Cannot check DRep deregistration refund** - Unable to verify the DRep refund amount due to missing context information


## 3. CollateralValidator (`collateral.rs`)

Validates collateral inputs and collateral return for script transactions.

### Errors (8)
- **Too many collateral inputs** - The number of collateral inputs exceeds the protocol maximum
- **No collateral inputs** - Transaction requires script execution but has no collateral inputs
- **Insufficient collateral** - The total collateral amount is less than required (percentage of transaction fee)
- **Incorrect total collateral field** - The declared total collateral doesn't match the sum of collateral input values
- **Calculated collateral contains non-ADA assets** - The collateral calculation results in non-ADA assets
- **Collateral input contains non-ADA assets** - One or more collateral inputs contain native tokens
- **Collateral is locked by script** - Collateral input is controlled by a script rather than a key
- **Collateral return too small** - The collateral return output doesn't meet minimum ADA requirements

### Warnings (3)
- **Collateral is unnecessary** - Transaction provides collateral but doesn't execute any scripts
- **Total collateral is not declared** - Collateral return is present but total collateral field is missing
- **Collateral input uses reward address** - Collateral input uses a reward address (unusual but not invalid)

---

## 4. FeeValidator (`fee.rs`)

Validates transaction fees against protocol parameters.

### Errors (1)
- **Fee too small** - The transaction fee is below the minimum required fee (calculated from tx size, execution units, and reference scripts)

### Warnings (1)
- **Fee is bigger than minimum fee** - The transaction fee is significantly higher than the minimum required (>10% over minimum)

---

## 5. OutputValidator (`output.rs`)

Validates transaction outputs for size and minimum ADA requirements.

### Errors (2)
- **Output too big** - A transaction output exceeds the maximum allowed size in bytes
- **Output too small** - A transaction output contains less ADA than the minimum required amount

---

## 6. RegistrationValidator (`registration.rs`)

Validates certificate-based registrations, deregistrations, and delegations.

### Errors (8)
- **Stake already registered** - Attempting to register an already registered stake key
- **Stake not registered** - Attempting to use an unregistered stake key for delegation or deregistration
- **Stake non-zero account balance** - Attempting to deregister a stake key with remaining rewards
- **Stake pool not registered** - Attempting to retire or update a non-existent stake pool
- **Wrong retirement epoch** - Pool retirement epoch is invalid (too early or too late)
- **Stake pool cost too low** - Pool cost parameter is below the minimum required
- **Committee is unknown** - Referencing a committee member that doesn't exist
- **Committee has previously resigned** - Attempting to authorize a committee member who has resigned

### Warnings (5)
- **Pool already registered** - Attempting to register an already registered pool
- **DRep already registered** - Attempting to register an already registered DRep
- **Committee already authorized** - Attempting to authorize an already authorized committee member
- **DRep not registered** - Certificate references a DRep that isn't registered
- **Duplicate registration in transaction** - Same entity is registered multiple times in one transaction
- **Duplicate committee cold resignation in transaction** - Same committee member resigns multiple times in one transaction
- **Duplicate committee hot registration in transaction** - Same committee hot key is registered multiple times in one transaction

---

## 7. TransactionLimitsValidator (`transaction_limits.rs`)

Validates transaction size, execution limits, and input validity.

### Errors (7)
- **Input set empty** - Transaction has no inputs
- **Maximum transaction size exceeded** - Transaction size in bytes exceeds protocol limit
- **Execution units too big** - Total execution units (memory/steps) exceed protocol limits
- **Reference scripts size too big** - Total size of reference scripts exceeds the limit
- **Outside validity interval** - Current slot is outside the transaction's validity interval
- **Bad inputs** - One or more inputs are already spent or don't exist
- **Reference input overlaps with input** - A reference input is also used as a regular input

### Warnings (1)
- **Inputs are not sorted** - Transaction inputs are not in canonical lexicographic order

---

## 8. WitnessValidator (`witness.rs`)

Validates cryptographic witnesses, signatures, and script execution requirements.

### Errors (10)
- **Missing verification key witnesses** - Required signatures are not provided
- **Invalid signature** - A provided signature is cryptographically invalid
- **Extraneous signature** - Unnecessary signatures are provided
- **Missing script witnesses** - Required scripts are not provided
- **Extraneous script witnesses** - Unnecessary scripts are provided in witness set
- **Native script is unsuccessful** - A native script evaluation fails
- **Missing redeemer** - Required redeemer for Plutus script is not provided
- **Missing datum** - Required datum for Plutus script is not provided
- **Extraneous datum witnesses** - Unnecessary datums are provided in witness set
- **Script data hash mismatch** - The script data hash doesn't match the calculated hash
