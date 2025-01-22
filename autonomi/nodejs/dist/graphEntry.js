"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GraphEntry = void 0;
class GraphEntry {
    constructor(nativeList) {
        this.nativeList = nativeList;
    }
    static async create(address) {
        // TODO: Implement native binding call
        throw new Error('Not implemented');
    }
    async get() {
        // TODO: Implement native binding call
        throw new Error('Not implemented');
    }
    async put(options, payment) {
        // TODO: Implement native binding call
        throw new Error('Not implemented');
    }
    async getCost(key) {
        // TODO: Implement native binding call
        throw new Error('Not implemented');
    }
}
exports.GraphEntry = GraphEntry;
