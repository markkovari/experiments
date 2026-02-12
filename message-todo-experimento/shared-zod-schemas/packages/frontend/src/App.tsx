import { useEffect, useState } from 'react';
import {
  TodoEndpoints,
  addTodoRequestSchema,
  buildPath,
  todoListResponseSchema,
  type Todo,
} from 'schemas';

function App() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [newText, setNewText] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editText, setEditText] = useState('');

  useEffect(() => {
    fetchTodos();
  }, []);

  const fetchTodos = async () => {
    const res = await fetch(TodoEndpoints.list.path);
    const data = await res.json();

    // Validate response with shared schema
    try {
      const validated = todoListResponseSchema.parse(data);
      setTodos(validated);
    } catch (error) {
      console.error('Invalid response data:', error);
    }
  };

  const addTodo = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newText.trim()) return;

    // Validate request with shared schema
    try {
      const requestData = addTodoRequestSchema.parse({ text: newText });

      await fetch(TodoEndpoints.add.path, {
        method: TodoEndpoints.add.method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestData),
      });
      setNewText('');
      fetchTodos();
    } catch (error) {
      console.error('Invalid input:', error);
    }
  };

  const toggleDone = async (id: string) => {
    await fetch(buildPath(TodoEndpoints.toggle.path, { id }), {
      method: TodoEndpoints.toggle.method,
    });
    fetchTodos();
  };

  const deleteTodo = async (id: string) => {
    await fetch(buildPath(TodoEndpoints.delete.path, { id }), {
      method: TodoEndpoints.delete.method,
    });
    fetchTodos();
  };

  const startEdit = (todo: Todo) => {
    setEditingId(todo.id);
    setEditText(todo.text);
  };

  const saveEdit = async (id: string) => {
    // Validate request with shared schema
    try {
      const requestData = addTodoRequestSchema.parse({ text: editText });

      await fetch(buildPath(TodoEndpoints.update.path, { id }), {
        method: TodoEndpoints.update.method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestData),
      });
      setEditingId(null);
      fetchTodos();
    } catch (error) {
      console.error('Invalid input:', error);
    }
  };

  return (
    <div style={{ padding: '20px', maxWidth: '600px', margin: '0 auto' }}>
      <h1>Todo App - Shared Zod Schemas</h1>

      <form onSubmit={addTodo} style={{ marginBottom: '20px' }}>
        <input
          type="text"
          value={newText}
          onChange={(e) => setNewText(e.target.value)}
          placeholder="Add new todo..."
          style={{ padding: '8px', width: '70%' }}
        />
        <button type="submit" style={{ padding: '8px 16px', marginLeft: '8px' }}>
          Add
        </button>
      </form>

      <ul style={{ listStyle: 'none', padding: 0 }}>
        {todos.map((todo) => (
          <li key={todo.id} style={{ marginBottom: '12px', display: 'flex', alignItems: 'center' }}>
            <input
              type="checkbox"
              checked={todo.isDone}
              onChange={() => toggleDone(todo.id)}
              style={{ marginRight: '8px' }}
            />
            {editingId === todo.id ? (
              <>
                <input
                  type="text"
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  style={{ padding: '4px', flex: 1 }}
                />
                <button type="button" onClick={() => saveEdit(todo.id)} style={{ marginLeft: '8px', padding: '4px 8px' }}>
                  Save
                </button>
                <button type="button" onClick={() => setEditingId(null)} style={{ marginLeft: '4px', padding: '4px 8px' }}>
                  Cancel
                </button>
              </>
            ) : (
              <>
                <span
                  style={{
                    flex: 1,
                    textDecoration: todo.isDone ? 'line-through' : 'none',
                    cursor: 'pointer',
                  }}
                  onClick={() => startEdit(todo)}
                  onKeyDown={(e) => e.key === 'Enter' && startEdit(todo)}
                  role="button"
                  tabIndex={0}
                >
                  {todo.text}
                </span>
                <button type="button" onClick={() => deleteTodo(todo.id)} style={{ marginLeft: '8px', padding: '4px 8px' }}>
                  Delete
                </button>
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;
