import { useState } from 'react';
import { trpc } from './trpc';

function App() {
  const [newText, setNewText] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editText, setEditText] = useState('');

  // tRPC hooks with full type safety!
  const { data: todos = [], refetch } = trpc.todos.list.useQuery();
  const addMutation = trpc.todos.add.useMutation({ onSuccess: () => refetch() });
  const toggleMutation = trpc.todos.toggle.useMutation({ onSuccess: () => refetch() });
  const updateMutation = trpc.todos.update.useMutation({ onSuccess: () => refetch() });
  const deleteMutation = trpc.todos.delete.useMutation({ onSuccess: () => refetch() });

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newText.trim()) return;
    await addMutation.mutateAsync({ text: newText });
    setNewText('');
  };

  const handleToggle = async (id: string) => {
    await toggleMutation.mutateAsync({ id });
  };

  const handleDelete = async (id: string) => {
    await deleteMutation.mutateAsync({ id });
  };

  const startEdit = (id: string, text: string) => {
    setEditingId(id);
    setEditText(text);
  };

  const saveEdit = async (id: string) => {
    await updateMutation.mutateAsync({ id, text: editText });
    setEditingId(null);
  };

  return (
    <div style={{ padding: '20px', maxWidth: '600px', margin: '0 auto' }}>
      <h1>Todo App - tRPC</h1>

      <form onSubmit={handleAdd} style={{ marginBottom: '20px' }}>
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
              onChange={() => handleToggle(todo.id)}
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
                  onClick={() => startEdit(todo.id, todo.text)}
                  onKeyDown={(e) => e.key === 'Enter' && startEdit(todo.id, todo.text)}
                  role="button"
                  tabIndex={0}
                >
                  {todo.text}
                </span>
                <button type="button" onClick={() => handleDelete(todo.id)} style={{ marginLeft: '8px', padding: '4px 8px' }}>
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
