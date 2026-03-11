import { useState } from 'react';

export function App() {
  const [count, setCount] = useState(0);

  return (
    <view
      h="full"
      w="full"
      flex="col"
      items="center"
      justify="center"
      p={2}
    >
      <text>Hello uzumaki</text>
      <p>Count: {count}</p>
      <button onClick={() => setCount(count + 1)}>Click me</button>
    </view>
  );
}
