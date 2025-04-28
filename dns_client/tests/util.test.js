const util = require('../util');
const axios = require('axios');

// Mock axios and network connector
jest.mock('axios');
jest.mock('../dns/network_connector', () => ({
  createNetworkConnector: jest.fn().mockReturnValue({
    connectToNetwork: jest.fn()
  })
}));

describe('Utility Functions', () => {
  // Reset mocks before each test
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('getJSONResponse', () => {
    it('should fetch and return JSON data successfully', async () => {
      // Mock successful response
      const mockData = { id: 'test-chain', bootNodes: ['/ip4/127.0.0.1/tcp/9944/ws'] };
      axios.get.mockResolvedValueOnce({ status: 200, data: mockData });

      const result = await util.getJSONResponse('http://example.com', 'spec.json');
      
      // Verify axios was called correctly
      expect(axios.get).toHaveBeenCalledWith('http://example.com/spec.json'.replace('json_server', 'localhost'));
      
      // Verify result
      expect(result).toEqual(mockData);
    });

    it('should handle errors gracefully', async () => {
      // Mock error response
      axios.get.mockRejectedValueOnce(new Error('Network error'));
      
      const result = await util.getJSONResponse('http://example.com', 'spec.json');
      
      // Verify axios was called
      expect(axios.get).toHaveBeenCalled();
      
      // Should return null on error
      expect(result).toBeNull();
    });

    it('should use the full address when fileName is null', async () => {
      // Mock successful response
      const mockData = { id: 'test-chain' };
      axios.get.mockResolvedValueOnce({ status: 200, data: mockData });

      const result = await util.getJSONResponse('http://example.com/spec.json');
      
      // Verify axios was called with the full address
      expect(axios.get).toHaveBeenCalledWith('http://example.com/spec.json'.replace('json_server', 'localhost'));
      
      // Verify result
      expect(result).toEqual(mockData);
    });
  });

  describe('getTLD', () => {
    it('should extract TLD from a domain', () => {
      expect(util.getTLD('example.com')).toBe('com');
      expect(util.getTLD('test.example.org')).toBe('org');
      expect(util.getTLD('sub.domain.net')).toBe('net');
    });
  });

  describe('getTLDSpec', () => {
    const mockApi = {
      query: {
        rootDNSModule: {
          tldMap: jest.fn()
        }
      }
    };

    const mockRootSpec = {
      id: 'root-chain',
      bootNodes: ['/ip4/127.0.0.1/tcp/9944/ws']
    };

    beforeEach(() => {
      // Reset the mock connector
      util.connector.connectToNetwork.mockReset();
    });

    it('should fetch TLD specification successfully', async () => {
      const mockTldResponse = {
        toHuman: () => ({
          chainSpec: JSON.stringify({
            id: 'test-tld-chain',
            bootNodes: ['/ip4/127.0.0.1/tcp/9945/ws']
          })
        })
      };

      // Mock successful connection and query
      util.connector.connectToNetwork.mockResolvedValueOnce(mockApi);
      mockApi.query.rootDNSModule.tldMap.mockResolvedValueOnce(mockTldResponse);

      const result = await util.getTLDSpec('com', mockRootSpec);

      // Verify connector was called with root spec
      expect(util.connector.connectToNetwork).toHaveBeenCalledWith(mockRootSpec);

      // Verify TLD query was made
      expect(mockApi.query.rootDNSModule.tldMap).toHaveBeenCalledWith('com');

      // Verify result
      expect(result).toEqual({
        id: 'test-tld-chain',
        bootNodes: ['/ip4/127.0.0.1/tcp/9945/ws']
      });
    });

    it('should handle connection errors', async () => {
      // Mock connection failure
      util.connector.connectToNetwork.mockRejectedValueOnce(new Error('Failed to connect'));

      await expect(util.getTLDSpec('com', mockRootSpec))
        .rejects
        .toThrow(/Failed to connect/);
    });

    it('should handle TLD query errors', async () => {
      // Mock successful connection but failed query
      util.connector.connectToNetwork.mockResolvedValueOnce(mockApi);
      mockApi.query.rootDNSModule.tldMap.mockRejectedValueOnce(new Error('Query failed'));

      await expect(util.getTLDSpec('com', mockRootSpec))
        .rejects
        .toThrow(/Query failed/);
    });
  });

  describe('getTargetSpec', () => {
    const mockApi = {
      query: {
        tldModule: {
          domainMap: jest.fn()
        }
      }
    };

    const mockTldSpec = {
      id: 'tld-chain',
      bootNodes: ['/ip4/127.0.0.1/tcp/9945/ws']
    };

    beforeEach(() => {
      // Reset the mock connector
      util.connector.connectToNetwork.mockReset();
    });

    it('should fetch target specification successfully', async () => {
      const mockDomainResponse = {
        toHuman: () => ({
          chainSpec: JSON.stringify({
            id: 'test-domain-chain',
            bootNodes: ['/ip4/127.0.0.1/tcp/9946/ws']
          })
        })
      };

      // Mock successful connection and query
      util.connector.connectToNetwork.mockResolvedValueOnce(mockApi);
      mockApi.query.tldModule.domainMap.mockResolvedValueOnce(mockDomainResponse);

      const result = await util.getTargetSpec('example.com', mockTldSpec);

      // Verify connector was called with TLD spec
      expect(util.connector.connectToNetwork).toHaveBeenCalledWith(mockTldSpec);

      // Verify domain query was made
      expect(mockApi.query.tldModule.domainMap).toHaveBeenCalledWith('example.com');

      // Verify result
      expect(result).toEqual({
        id: 'test-domain-chain',
        bootNodes: ['/ip4/127.0.0.1/tcp/9946/ws']
      });
    });

    it('should handle connection errors', async () => {
      // Mock connection failure
      util.connector.connectToNetwork.mockRejectedValueOnce(new Error('Failed to connect'));

      await expect(util.getTargetSpec('example.com', mockTldSpec))
        .rejects
        .toThrow(/Failed to connect/);
    });

    it('should handle domain query errors', async () => {
      // Mock successful connection but failed query
      util.connector.connectToNetwork.mockResolvedValueOnce(mockApi);
      mockApi.query.tldModule.domainMap.mockRejectedValueOnce(new Error('Query failed'));

      await expect(util.getTargetSpec('example.com', mockTldSpec))
        .rejects
        .toThrow(/Query failed/);
    });
  });
});
