import "./App.css";
import { useQuery } from "@tanstack/react-query";
import { getTodo, getTodos } from "./api/todos";
import { FormEvent, useState } from "react";

function App() {
	const [id, setId] = useState(0);
	const { data, isLoading, error, isPending } = useQuery({
		queryKey: ["todos"],
		queryFn: getTodos,
	});

	const { data: todoWithId } = useQuery({
		queryKey: ["todos", id],
		queryFn: () => getTodo(id),
	});

	const getByIdSubmit = (e: FormEvent) => {
		e.preventDefault();
		e.stopPropagation();
	};

	if (isLoading) {
		return <p>Loading ....</p>;
	}

	if (isPending) {
		return <p>Almost there ....</p>;
	}

	if (error) {
		return <p>Ups :( {JSON.stringify(error)}</p>;
	}

	return (
		<>
			<h1>First</h1>
			<p>{JSON.stringify(data)}</p>
			<h1>By id</h1>
			<form onSubmit={getByIdSubmit}>
				<label htmlFor="todo-id">Id:</label>
				<input
					onChange={({ target }) => setId(+target.value)}
					id="todo-id"
					type="number"
				/>
				<button type="submit">Get</button>
			</form>
			<p>WithID : {JSON.stringify(todoWithId)}</p>
		</>
	);
}

export default App;
