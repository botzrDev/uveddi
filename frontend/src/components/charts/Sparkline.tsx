import { Line, LineChart, ResponsiveContainer } from 'recharts';

export default function Sparkline({ data, color = '#7c3aed' }: { data: number[]; color?: string }) {
  const points = data.map((v, i) => ({ x: i, y: v }));
  return (
    <div style={{ width: 100, height: 32 }}>
      <ResponsiveContainer>
        <LineChart data={points} margin={{ top: 2, right: 4, left: 4, bottom: 2 }}>
          <Line type="monotone" dataKey="y" stroke={color} strokeWidth={2} dot={false} />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
