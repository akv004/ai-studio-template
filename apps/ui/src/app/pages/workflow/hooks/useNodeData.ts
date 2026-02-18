import { useCallback } from 'react';
import { useReactFlow } from '@xyflow/react';

export function useNodeData(nodeId: string) {
    const { setNodes } = useReactFlow();

    const updateField = useCallback((field: string, value: unknown) => {
        setNodes(nds => nds.map(n =>
            n.id === nodeId ? { ...n, data: { ...n.data, [field]: value } } : n
        ));
    }, [nodeId, setNodes]);

    const updateFields = useCallback((fields: Record<string, unknown>) => {
        setNodes(nds => nds.map(n =>
            n.id === nodeId ? { ...n, data: { ...n.data, ...fields } } : n
        ));
    }, [nodeId, setNodes]);

    return { updateField, updateFields };
}
