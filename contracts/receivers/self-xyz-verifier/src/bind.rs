alloy::sol!(
    struct DecodedUserData {
        uint8 action;
        bytes32 configId;
    }

    struct GenericDiscloseOutputV2 {
        bytes32 attestationId;                    // E_PASSPORT or EU_ID_CARD
        uint256 userIdentifier;                   // User's unique identifier
        uint256 nullifier;                        // Anti-replay nullifier
        uint256[4] forbiddenCountriesListPacked;  // Forbidden countries used

        // Disclosed identity information
        string issuingState;                      // Document issuing country
        string[] name;                            // [first, middle, last] names
        string idNumber;                          // Passport/ID number
        string nationality;                       // User's nationality
        string dateOfBirth;                       // Birth date (DD-MM-YY)
        string gender;                            // User's gender
        string expiryDate;                        // Document expiry date

        // Verification results
        uint256 olderThan;                        // Verified minimum age
        bool[3] ofac;                             // OFAC results [passport, name+dob, name+yob]
    }
);
