const { getTLD, getTLDSpec, connectToNetwork, getJSONResponse, connector } = require("../util");
const { TxType } = require('../util');
const { createResolver } = require("./resolver");
const { createTransaction } = require("../transaction/transaction");

/**
 * Registry class provides methods to interact with blockchain networks
 * for registering assets, TLDs, and domains.
 */
class Registry {
    /**
     * Constructs a new Registry instance.
     * 
     * @param {string} rootNetworkSpecUrl - URL to the root network's chain specification.
     * @param {string} phrase - Mnemonic phrase used for signing transactions.
     */
    constructor(rootNetworkSpecUrl, phrase) {
        this.rootNetworkSpecUrl = rootNetworkSpecUrl;
        this.phrase = phrase;
        this.rootSpec = null; // Holds the chain specification of the root network.
    }

    /**
     * Initializes the Registry by fetching the root network's chain specification.
     * 
     * @returns {Promise<void>}
     * @throws {Error} If fetching the chain specification fails.
     */
    async init() {
        this.rootSpec = await getJSONResponse(this.rootNetworkSpecUrl);
    }

    /**
     * Registers a new asset on a blockchain associated with a specific domain.
     * 
     * @param {string} domain - Domain representing the blockchain network.
     * @param {string} assetId - Unique identifier for the asset.
     * @param {number} amount - Minimum balance required for the asset.
     * @returns {Promise<void>}
     * @throws {Error} If the domain network connection or transaction fails.
     */
    async registerAsset(domain, assetId, amount) {
        // Create a resolver to find the chain specification for the domain.
        let resolver = await createResolver(this.rootNetworkSpecUrl);
        await resolver.init();

        // Resolve the chain specification for the domain.
        let chainSpec = await resolver.resolve(domain);

        // Connect to the domain's blockchain network.
        let api = await connector.connectToNetwork(chainSpec.targetSpec);

        // Create and send the transaction to register the asset.
        await createTransaction(TxType.TX_ASSET_CREATE, {
            assetId: assetId,
            minBalance: amount
        }, api, this.phrase).sendTransaction();
    }

    /**
     * Registers a new TLD (Top-Level Domain) on the root network.
     * 
     * @param {string} tld - The TLD to register.
     * @param {object} tldSpec - Chain specification for the TLD.
     * @returns {Promise<void>}
     * @throws {Error} If the root network connection or transaction fails.
     */
    async registerTLD(tld, tldSpec) {
        try {
            // Connect to the root network.
            let api = await connector.connectToNetwork(this.rootSpec);

            // Create and send the transaction to register the TLD.
            await createTransaction(TxType.TX_ROOT, {
                target: tld,
                targetSpec: tldSpec
            }, api, this.phrase).sendTransaction();
        } catch (error) {
            // Handle specific connection errors or propagate other errors.
            if (error.toString() === "CONNECTION_ERROR") {
                throw new Error("Could not connect to the root network.");
            } else {
                throw error;
            }
        }
    }

    /**
     * Registers a new domain under a specified TLD.
     * 
     * @param {string} domain - The domain to register.
     * @param {object} domainSpec - Chain specification for the domain.
     * @returns {Promise<void>}
     * @throws {Error} If the TLD network connection or transaction fails.
     */
    async registerDomain(domain, domainSpec) {
        try {
            // Extract the TLD from the domain.
            let tld = getTLD(domain);

            // Fetch the chain specification for the TLD.
            let tldSpec = await getTLDSpec(tld, this.rootSpec);

            // Connect to the TLD's blockchain network.
            let api = await connector.connectToNetwork(tldSpec);

            // Create and send the transaction to register the domain.
            await createTransaction(TxType.TX_TLD, {
                target: domain,
                targetSpec: domainSpec
            }, api, this.phrase).sendTransaction();
        } catch (error) {
            // Handle specific connection errors or propagate other errors.
            if (error.toString() === "CONNECTION_ERROR") {
                throw new Error("Could not connect to the TLD network.");
            } else {
                throw error;
            }
        }
    }
}

/**
 * Factory function to create a new Registry instance.
 * 
 * @param {string} rootSpecAddr - URL of the root network's chain specification.
 * @param {string} phrase - Mnemonic phrase for signing transactions.
 * @returns {Registry} A new Registry instance.
 */
exports.createRegistry = (rootSpecAddr, phrase) => {
    return new Registry(rootSpecAddr, phrase);
};
