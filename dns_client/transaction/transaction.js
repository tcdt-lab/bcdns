const {TxType} = require('../util');
const {Keyring} = require("@polkadot/api");
const fs = require('fs');
const path = require('path');

class Transaction {

    constructor(txType, txArgs, api, phrase) {
        this.txType = txType;
        this.txArgs = txArgs;
        this.api = api;
        this.phrase = phrase;
    }

    async sendTransaction() {
        // Init credentials
        const keyring = new Keyring({type: 'sr25519'});
        const account = keyring.addFromUri(this.phrase);
        let tx;

        try {
            switch (this.txType) {
                case TxType.TX_ROOT:
                    let tldName = this.api.createType('Vec<u8>', this.txArgs.target);
                    // Read, parse and filter JSON file for targetSpec
                    const tldSpecData = JSON.parse(fs.readFileSync(path.resolve(this.txArgs.targetSpec), 'utf8'));
                    const filteredTldSpec = {
                        name: tldSpecData.name,
                        id: tldSpecData.id,
                        chainType: tldSpecData.chainType,
                        bootNodes: tldSpecData.bootNodes
                    };
                    let tldSpec = this.api.createType('Vec<u8>', JSON.stringify(filteredTldSpec));
                    tx = this.api.tx.rootDNSModule.registerTld(tldName, tldSpec);
                    break;
                case TxType.TX_TLD:
                    let domainName = this.api.createType('Vec<u8>', this.txArgs.target);
                    // Read, parse and filter JSON file for targetSpec
                    const networkSpecData = JSON.parse(fs.readFileSync(path.resolve(this.txArgs.targetSpec), 'utf8'));
                    const filteredNetworkSpec = {
                        name: networkSpecData.name,
                        id: networkSpecData.id,
                        chainType: networkSpecData.chainType,
                        bootNodes: networkSpecData.bootNodes
                    };
                    let networkSpec = this.api.createType('Vec<u8>', JSON.stringify(filteredNetworkSpec));
                    let maintainer = this.api.createType('Vec<u8>', "maintainer"); // TODO: hardcoded for now
                    tx = this.api.tx.tldModule.registerDomain(domainName, networkSpec, maintainer);
                    break;
                case TxType.TX_ASSET_CREATE:
                    let assetId = this.api.createType('u32', this.txArgs.assetId);
                    let owner = this.api.createType('MultiAddress', account.address);
                    let minBalance = this.api.createType('u128', this.txArgs.minBalance);
                    tx = this.api.tx.assetsModule.create(assetId, owner, minBalance);
                    break;
                default:
                    throw new Error("Invalid transaction type.");
            }
            
            // Get current nonce and increment by 1
            const nonce = await this.api.rpc.system.accountNextIndex(account.address);
            await tx.signAndSend(account, { nonce });
        } catch (error) {
            throw error;
        }
    }
}

exports.createTransaction = (txType, txArgs, api, phrase) => {
    return new Transaction(txType, txArgs, api, phrase);
};