const NetworkConnector = require('../dns/network_connector');
const { polkadotConnect } = require('../polkadot/connector');

// Mock the polkadotConnect function
jest.mock('../polkadot/connector', () => ({
  polkadotConnect: jest.fn()
}));

describe('NetworkConnector', () => {
  let connector;
  const mockApi = {
    query: {
      system: {
        events: jest.fn()
      }
    }
  };
  const mockSpec = {
    id: 'test-network',
    bootNodes: [
      '/ip4/127.0.0.1/tcp/9944/ws',
      '/ip4/127.0.0.1/tcp/9945/ws'
    ]
  };

  beforeEach(() => {
    // Create a fresh connector instance before each test
    connector = NetworkConnector.createNetworkConnector();
    // Clear all mocks
    jest.clearAllMocks();
    // Reset polkadotConnect mock
    polkadotConnect.mockReset();
  });

  describe('connectToNetwork', () => {
    it('should connect to a network successfully', async () => {
      // Mock successful connection
      polkadotConnect.mockResolvedValueOnce(mockApi);

      const api = await connector.connectToNetwork(mockSpec);

      // Verify polkadotConnect was called with correct address format
      expect(polkadotConnect.mock.calls[0][0]).toMatch(/^ws:\/\/127\.0\.0\.1:\d{4}$/);
      
      // Verify the API was returned
      expect(api).toBe(mockApi);
    });

    it('should use cached connection for the same network', async () => {
      // First connection
      polkadotConnect.mockResolvedValueOnce(mockApi);
      await connector.connectToNetwork(mockSpec);

      // Second connection to same network
      const cachedApi = await connector.connectToNetwork(mockSpec);

      // Verify polkadotConnect was only called once
      expect(polkadotConnect).toHaveBeenCalledTimes(1);
      
      // Verify cached API was returned
      expect(cachedApi).toBe(mockApi);
    });

    it('should try multiple boot nodes until successful connection', async () => {
      // Mock connection attempts
      polkadotConnect
        .mockRejectedValueOnce(new Error('Failed to connect'))
        .mockResolvedValueOnce(mockApi);

      const api = await connector.connectToNetwork(mockSpec);

      // Verify polkadotConnect was called multiple times
      expect(polkadotConnect).toHaveBeenCalledTimes(2);
      
      // Verify successful connection was returned
      expect(api).toBe(mockApi);
    });

    it('should throw error if no connection can be established', async () => {
      // Mock all connection attempts failing
      polkadotConnect.mockRejectedValue(new Error('Failed to connect'));

      // Expect the connection attempt to throw an error
      await expect(connector.connectToNetwork(mockSpec))
        .rejects
        .toThrow(/Failed to connect/);
    });
  });
});
