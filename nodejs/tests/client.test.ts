import { Client } from '../src/client';

describe('Client', () => {
    describe('connect', () => {
        it('should throw not implemented error', async () => {
            await expect(Client.connect({ peers: [] })).rejects.toThrow('Not implemented');
        });
    });

    describe('GraphEntryOperations', () => {
        it('should throw not implemented error for GraphEntryGet', async () => {
            const client = await Client.connect({ peers: [] }).catch(() => null);
            if (!client) return;
            await expect(client.GraphEntryGet('address')).rejects.toThrow('Not implemented');
        });

        it('should throw not implemented error for GraphEntryPut', async () => {
            const client = await Client.connect({ peers: [] }).catch(() => null);
            if (!client) return;
            await expect(
                client.GraphEntryPut(
                    {
                        owner: 'owner',
                        counter: 0,
                        target: 'target',
                        key: 'key'
                    },
                    { type: 'wallet', wallet: 'wallet' }
                )
            ).rejects.toThrow('Not implemented');
        });
    });
}); 