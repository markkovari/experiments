import { randomUUID, type UUID } from "node:crypto";
import { RedisCache } from "./keyVal.js";

export type BatchId = UUID;
export type Status = "success" | "error" | "inprogress";
export type BatchStatus = Status | "partialsuccess";
export type Id = UUID;

export type Envelope<T> = {
	batchID: BatchId;
	id: Id;
	status: Status;
	payload: T;
};

export type Batch<T> = {
	id: BatchId;
	elements: Envelope<T>[];
	status: BatchStatus;
};

export const createBatch = async <T>(elements: T[]): Promise<Batch<T>> => {
	if (elements.length === 0) {
		throw new Error("denifintely should not create empty batches");
	}
	const client = await RedisCache.getClient();
	const id = randomUUID();
	const batch: Batch<T> = {
		elements: elements.map((payload) => ({
			id: randomUUID(),
			payload,
			batchID: id,
			status: "inprogress",
		})),
		id,
		status: "inprogress",
	};
	await client.set(id, JSON.stringify(batch));
	return batch;
};

export const updateBatchWithJobStatus = async <T>(
	batchId: UUID,
	jobId: UUID,
	status: Status,
) => {
	const client = await RedisCache.getClient();
	const oldValue = await client.get(batchId);
	if (!oldValue) {
		throw new Error("You need to create the batch first");
	}
	const batchValue = JSON.parse(oldValue.toString()) as Batch<T>;
	const { elements } = batchValue;
	console.log({ elements });
	const newElements = elements.map((element) => ({
		...element,
		status: element.id === jobId ? status : element.status,
	}));
	console.log({ newElements });
	const jobStatuses = newElements.map(({ status }) => status);
	// if any of these in progress the batch is in progress too
	const inProgress = jobStatuses.some((s) => s === "inprogress");
	if (inProgress) {
		return await client.set(
			batchId,
			JSON.stringify({
				...batchValue,
				elements: newElements,
				status: "inprogress",
			} as Batch<T>),
		);
	}
	const elementsLength = elements.length;
	const errors = jobStatuses.filter((s) => s === "error");
	let newBatchStatus: BatchStatus = "inprogress";
	if (errors.length === 0) {
		newBatchStatus = "success";
	} else if (errors.length !== 0 && errors.length !== elementsLength) {
		newBatchStatus = "partialsuccess";
	} else {
		newBatchStatus = "error";
	}
	return await client.set(
		batchId,
		JSON.stringify({
			...batchValue,
			elements: newElements,
			status: newBatchStatus,
		} as Batch<T>),
	);
};
