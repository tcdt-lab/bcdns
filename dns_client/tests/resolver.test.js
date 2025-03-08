const { createResolver } = require('../dns/resolver');
const util = require('../util');

// Mock the utility functions
jest.mock('../util', () => ({
  getJSONResponse: jest.fn(),
  getTLDSpec: jest.fn(),
  getTargetSpec: jest.fn(),
  getTLD: jest.fn(),
  connector: {
    connectToNetwork: jest.fn()
  }
}));

describe('DNSResolver', () => {
  let resolver;
  const mockRootSpecUrl = 'http://example.com/root-spec.json';
  const mockRootSpec = {
    id: 'root-chain',
    bootNodes: ['/ip4/127.0.0.1/tcp/9944/ws']
  };
  const mockTldSpec = {
    id: 'tld-chain',
    bootNodes: ['/ip4/127.0.0.1/tcp/9945/ws']
  };
  const mockTargetSpec = {
    id: 'target-chain',
    bootNodes: ['/ip4/127.0.0.1/tcp/9946/ws']
  };
  const mockApi = {
    query: {
      assetsModule: {
        asset: jest.fn()
      }
    }
  };

  beforeEach(async () => {
    // Reset all mocks
    jest.clearAllMocks();
    
    // Create a fresh resolver instance
    resolver = createResolver(mockRootSpecUrl);
    
    // Mock successful root spec fetch
    util.getJSONResponse.mockResolvedValue(mockRootSpec);
    
    // Initialize resolver
    await resolver.init();
  });

  describe('init', () => {
    it('should fetch and store root specification', async () => {
      // Verify getJSONResponse was called with correct URL
      expect(util.getJSONResponse).toHaveBeenCalledWith(mockRootSpecUrl);
      
      // Verify root spec was stored
      expect(resolver.rootSpec).toEqual(mockRootSpec);
    });

    it('should handle initialization errors', async () => {
      // Create new resolver for this test
      const errorResolver = createResolver(mockRootSpecUrl);
      
      // Mock failed root spec fetch
      util.getJSONResponse.mockRejectedValueOnce(new Error('Failed to fetch'));
      
      // Expect initialization to fail
      await expect(errorResolver.init()).rejects.toThrow();
    });
  });

  describe('resolveAsset', () => {
    const mockAssetId = '123';
    const mockAssetDetails = {
      name: 'Test Asset',
      owner: 'test-owner',
      toHuman: () => ({
        name: 'Test Asset',
        owner: 'test-owner'
      })
    };

    beforeEach(() => {
      // Mock successful network connection
      util.connector.connectToNetwork.mockResolvedValue(mockApi);
      
      // Mock successful asset query
      mockApi.query.assetsModule.asset.mockResolvedValue(mockAssetDetails);
    });

    it('should resolve asset details successfully', async () => {
      const result = await resolver.resolveAsset(mockTargetSpec, mockAssetId);
      
      // Verify network connection was attempted
      expect(util.connector.connectToNetwork).toHaveBeenCalledWith(mockTargetSpec);
      
      // Verify asset query was made
      expect(mockApi.query.assetsModule.asset).toHaveBeenCalledWith(mockAssetId);
      
      // Verify human-readable result
      expect(result).toEqual({
        name: 'Test Asset',
        owner: 'test-owner'
      });
    });

    it('should handle asset resolution errors', async () => {
      // Mock failed asset query
      mockApi.query.assetsModule.asset.mockRejectedValue(new Error('Query failed'));
      
      // Expect asset resolution to fail
      await expect(resolver.resolveAsset(mockTargetSpec, mockAssetId))
        .rejects
        .toThrow();
    });
  });

  describe('resolve', () => {
    beforeEach(() => {
      // Mock utility functions
      util.getTLD.mockReturnValue('com');
      util.getTLDSpec.mockResolvedValue(mockTldSpec);
      util.getTargetSpec.mockResolvedValue(mockTargetSpec);
    });

    it('should resolve domain without asset', async () => {
      const result = await resolver.resolve('example.com');
      
      // Verify TLD extraction
      expect(util.getTLD).toHaveBeenCalledWith('example.com');
      
      // Verify TLD spec resolution
      expect(util.getTLDSpec).toHaveBeenCalledWith('com', mockRootSpec);
      
      // Verify target spec resolution
      expect(util.getTargetSpec).toHaveBeenCalledWith('example.com', mockTldSpec);
      
      // Verify result format
      expect(result).toEqual({ targetSpec: mockTargetSpec });
    });

    it('should resolve domain with asset', async () => {
      const mockAssetDetails = {
        name: 'Test Asset',
        owner: 'test-owner'
      };
      
      // Mock successful asset resolution
      util.connector.connectToNetwork.mockResolvedValue(mockApi);
      mockApi.query.assetsModule.asset.mockResolvedValue({
        toHuman: () => mockAssetDetails
      });

      const result = await resolver.resolve('example.com/asset/123');
      
      // Verify domain resolution occurred
      expect(util.getTLD).toHaveBeenCalledWith('example.com');
      expect(util.getTLDSpec).toHaveBeenCalledWith('com', mockRootSpec);
      expect(util.getTargetSpec).toHaveBeenCalledWith('example.com', mockTldSpec);
      
      // Verify asset resolution
      expect(result).toEqual({ asset: mockAssetDetails });
    });

    it('should handle resolution errors', async () => {
      // Mock failed TLD spec resolution
      util.getTLDSpec.mockRejectedValue(new Error('TLD resolution failed'));
      
      // Expect domain resolution to fail
      await expect(resolver.resolve('example.com'))
        .rejects
        .toThrow();
    });
  });
});
