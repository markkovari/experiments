import { gql, useMutation, useQuery } from '@apollo/client';
import { useState } from 'react';

const GET_TODOS = gql`
  query GetTodos {
    todos {
      id
      text
      isDone
    }
  }
`;

const ADD_TODO = gql`
  mutation AddTodo($text: String!) {
    addTodo(text: $text) {
      id
      text
      isDone
    }
  }
`;

const TOGGLE_TODO = gql`
  mutation ToggleTodo($id: ID!) {
    toggleTodo(id: $id) {
      id
      text
      isDone
    }
  }
`;

const UPDATE_TODO = gql`
  mutation UpdateTodo($id: ID!, $text: String!) {
    updateTodo(id: $id, text: $text) {
      id
      text
      isDone
    }
  }
`;

const DELETE_TODO = gql`
  mutation DeleteTodo($id: ID!) {
    deleteTodo(id: $id)
  }
`;

function App() {
  const { loading, error, data, refetch } = useQuery(GET_TODOS);
  const [addTodo] = useMutation(ADD_TODO, { onCompleted: () => refetch() });
  const [toggleTodo] = useMutation(TOGGLE_TODO, { onCompleted: () => refetch() });
  const [updateTodo] = useMutation(UPDATE_TODO, { onCompleted: () => refetch() });
  const [deleteTodo] = useMutation(DELETE_TODO, { onCompleted: () => refetch() });

  const [newText, setNewText] = useState('');
  const [editingId, setEditingId] = useState(null);
  const [editText, setEditText] = useState('');

  if (loading) return <div style={{ padding: '20px' }}>Loading...</div>;
  if (error) return <div style={{ padding: '20px' }}>Error: {error.message}</div>;

  const handleAdd = async (e) => {
    e.preventDefault();
    if (!newText.trim()) return;
    await addTodo({ variables: { text: newText } });
    setNewText('');
  };

  const handleToggle = async (id) => {
    await toggleTodo({ variables: { id } });
  };

  const handleDelete = async (id) => {
    await deleteTodo({ variables: { id } });
  };

  const startEdit = (todo) => {
    setEditingId(todo.id);
    setEditText(todo.text);
  };

  const saveEdit = async (id) => {
    await updateTodo({ variables: { id, text: editText } });
    setEditingId(null);
  };

  return (
    <div style={{ padding: '20px', maxWidth: '600px', margin: '0 auto' }}>
      <h1>Todo App - GraphQL</h1>

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
        {data.todos.map((todo) => (
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
                    cursor: 'pointer'
                  }}
                  onClick={() => startEdit(todo)}
                  onKeyDown={(e) => e.key === 'Enter' && startEdit(todo)}
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
