import { createTRPCReact } from '@trpc/react-query';
import type { AppRouter } from 'api';

// Create tRPC React hooks with full type safety from the backend
export const trpc = createTRPCReact<AppRouter>();
