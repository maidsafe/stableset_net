import { GraphEntryOptions, PaymentOption } from './types';
export declare class GraphEntry {
    private nativeList;
    private constructor();
    static create(address: string): Promise<GraphEntry>;
    get(): Promise<any[]>;
    put(options: GraphEntryOptions, payment: PaymentOption): Promise<void>;
    getCost(key: string): Promise<string>;
}
