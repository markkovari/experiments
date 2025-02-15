const getTodos = async () => {
	const response = await fetch("https://jsonplaceholder.typicode.com/todos/1");
	const todos = await response.json();
	return todos;
};

const getTodo = async (id: number) => {
	const response = await fetch(
		`https://jsonplaceholder.typicode.com/todos/${id}`,
	);
	const todos = await response.json();
	return todos;
};

export { getTodos, getTodo };
