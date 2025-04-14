const express = require('express');
const path = require('path');
const { createResolver } = require('../../dns/resolver');
const { ROOT_DNS_NETWORK_SPEC_ADDR } = require('../../config');

class WebServer {
    constructor() {
        this.app = express();
        this.resolver = createResolver(ROOT_DNS_NETWORK_SPEC_ADDR);
        this.setupMiddleware();
        this.setupRoutes();
    }

    async init() {
        await this.resolver.init();
    }

    setupMiddleware() {
        this.app.use(express.json());
        this.app.use(express.static(path.join(__dirname, '../public')));
    }

    setupRoutes() {
        this.app.post('/resolve/domain', async (req, res) => {
            try {
                const { domain } = req.body;
                const result = await this.resolver.resolve(domain);
                res.json(result);
            } catch (error) {
                res.status(500).json({ error: error.message });
            }
        });

        this.app.post('/resolve/asset', async (req, res) => {
            try {
                const { asset } = req.body;
                const result = await this.resolver.resolveAsset(asset);
                res.json(result);
            } catch (error) {
                res.status(500).json({ error: error.message });
            }
        });
    }

    start(port = 3000) {
        this.app.listen(port, () => {
            console.log(`Web server running at http://localhost:${port}`);
        });
    }
}

module.exports = { WebServer };
