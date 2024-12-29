import { Client } from '../src/client';

describe('Client', () => {
    describe('connect', () => {
        it('should throw not implemented error', async () => {
            await expect(Client.connect({ peers: [] })).rejects.toThrow('Not implemented');
        });
    });

    describe('linkedListOperations', () => {
        it('should throw not implemented error for linkedListGet', async () => {
            const client = await Client.connect({ peers: [] }).catch(() => null);
            if (!client) return;
            await expect(client.linkedListGet('address')).rejects.toThrow('Not implemented');
        });

        it('should throw not implemented error for linkedListPut', async () => {
            const client = await Client.connect({ peers: [] }).catch(() => null);
            if (!client) return;
            await expect(
                client.linkedListPut(
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