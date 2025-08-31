/**
 * @description Defines the possible statuses of a processing batch.
 */
export enum BatchStatus {
	Pending = "pending",
	InProgress = "in_progress",
	Completed = "completed",
	Failed = "failed",
}

/**
 * @description The state object for a batch, stored in the KV bucket.
 * @field status - The current status of the batch.
 * @field totalItems - The total number of items in the batch.
 * @field completedItems - The number of items successfully processed.
 * @field failedItems - A list of item IDs that failed processing.
 * @field createdAt - The timestamp when the batch was created.
 */
export interface BatchState {
	status: BatchStatus;
	totalItems: number;
	completedItems: number;
	failedItems: string[];
	createdAt: number;
}
