const { polkadotConnect } = require("../polkadot/connector");

/**
 * Manages connections to blockchain networks using the Polkadot API.
 * Caches API connections to avoid repeated network calls.
 */
class NetworkConnector {
    constructor() {
        this.apiCache = {}; // Cache for storing connected network APIs
    }

    /**
     * Connects to a specified network using its chain specification.
     * If a cached connection exists, it is reused.
     * 
     * @param {object} networkSpec - The specification of the network to connect to.
     * @returns {Promise<object>} The API instance connected to the network.
     * @throws {Error} If no connection can be established.
     */
    async connectToNetwork(networkSpec) {
        let networkId = this.#extractNetworkId(networkSpec); // Extracts network ID
        let bootNodeList = this.#extractBootNodesFromSpecJson(networkSpec); // Extracts boot nodes

        // Use cached API if available
        if (this.apiCache[networkId]) {
            console.log(`Using cached API for network ${networkId}`);
            return this.apiCache[networkId];
        }

        let api = null;
        // Try connecting to a random boot node until successful
        while (!api) {
            let node = bootNodeList[Math.floor(Math.random() * bootNodeList.length)];
            console.log(`Trying to connect to ${node}`);
            api = await polkadotConnect(this.#getConnectionAddress(node)); // Attempt connection
        }

        if (!api) {
            throw new Error("CONNECTION_ERROR"); // Throw error if no connection is made
        }

        this.apiCache[networkId] = api; // Cache the successful API connection
        return api;
    }

    /**
     * Constructs a connection address from the boot node's multi-address.
     * 
     * @param {string} bootNodeMPAddr - The boot node multi-address.
     * @returns {string} The connection address in `http://<addr>:<port>` format.
     */
    #getConnectionAddress(bootNodeMPAddr) {
        let addrSpl = bootNodeMPAddr.split('/');
        let addr = addrSpl[2]; // Extracts the address part
        let port = addrSpl[4]; // Extracts the port part
        return `http://${addr}:${port}`; // Constructs the connection address
    }

    /**
     * Extracts the boot nodes from the network's chain specification.
     * 
     * @param {object} specJson - The network's chain specification.
     * @returns {Array<string>} The list of boot nodes.
     * @throws {Error} If no boot nodes are found.
     */
    #extractBootNodesFromSpecJson(specJson) {
        if (specJson && specJson.bootNodes) {
            return specJson.bootNodes; // Return the boot nodes if available
        } else {
            throw new Error(`Boot nodes not found for chainSpec: ${specJson}`);
        }
    }

    /**
     * Extracts the network ID from the network's chain specification.
     * 
     * @param {object} specJson - The network's chain specification.
     * @returns {string} The network ID.
     * @throws {Error} If no network ID is found.
     */
    #extractNetworkId(specJson) {
        if (specJson && specJson.id) {
            return specJson.id; // Return the network ID if available
        } else {
            throw new Error(`Network ID not found for chainSpec: ${specJson}`);
        }
    }
}

/**
 * Factory function to create a new NetworkConnector instance.
 * @returns {NetworkConnector} A new instance of the NetworkConnector class.
 */
exports.createNetworkConnector = () => {
    return new NetworkConnector();
};
