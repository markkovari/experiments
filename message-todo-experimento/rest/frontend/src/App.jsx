import { useEffect, useState } from 'react';

function App() {
  const [todos, setTodos] = useState([]);
  const [newText, setNewText] = useState('');
  const [editingId, setEditingId] = useState(null);
  const [editText, setEditText] = useState('');

  useEffect(() => {
    fetchTodos();
  }, []);

  const fetchTodos = async () => {
    const res = await fetch('/todos');
    const data = await res.json();
    setTodos(data);
  };

  const addTodo = async (e) => {
    e.preventDefault();
    if (!newText.trim()) return;
    await fetch('/todos', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text: newText })
    });
    setNewText('');
    fetchTodos();
  };

  const toggleDone = async (id) => {
    await fetch(`/todos/${id}/toggle`, { method: 'PATCH' });
    fetchTodos();
  };

  const deleteTodo = async (id) => {
    await fetch(`/todos/${id}`, { method: 'DELETE' });
    fetchTodos();
  };

  const startEdit = (todo) => {
    setEditingId(todo.id);
    setEditText(todo.text);
  };

  const saveEdit = async (id) => {
    await fetch(`/todos/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text: editText })
    });
    setEditingId(null);
    fetchTodos();
  };

  return (
    <div style={{ padding: '20px', maxWidth: '600px', margin: '0 auto' }}>
      <h1>Todo App</h1>

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
                <button onClick={() => saveEdit(todo.id)} style={{ marginLeft: '8px', padding: '4px 8px' }}>
                  Save
                </button>
                <button onClick={() => setEditingId(null)} style={{ marginLeft: '4px', padding: '4px 8px' }}>
                  Cancel
                </button>
              </>
            ) : (
              <>
                <span
                  style={{
                    flex: 1,
                    textDecoration: todo.isDone ? 'line-through' : 'none',
                    cursor: 'pointer'
                  }}
                  onClick={() => startEdit(todo)}
                >
                  {todo.text}
                </span>
                <button onClick={() => deleteTodo(todo.id)} style={{ marginLeft: '8px', padding: '4px 8px' }}>
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
