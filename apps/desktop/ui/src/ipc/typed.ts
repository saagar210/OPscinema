import { bindGeneratedClient } from './generated';
import { client } from './client';

export const ipc = bindGeneratedClient(client);
