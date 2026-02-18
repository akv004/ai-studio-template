import { useAppStore } from '../../state/store';
import type { CreateWorkflowRequest } from '@ai-studio/shared';
import { WorkflowList } from './workflow/WorkflowList';
import { WorkflowCanvas } from './workflow/WorkflowCanvas';

export function NodeEditorPage() {
    const { fetchWorkflow, createWorkflow, selectedWorkflow, setSelectedWorkflow, addToast } = useAppStore();

    const handleSelectWorkflow = async (id: string) => {
        await fetchWorkflow(id);
    };

    const handleCreate = async () => {
        const req: CreateWorkflowRequest = {
            name: `Workflow ${new Date().toLocaleDateString()}`,
            description: '',
        };
        const workflow = await createWorkflow(req);
        setSelectedWorkflow(workflow);
    };

    const handleCreateFromTemplate = async (templateId: string) => {
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            const graphJson = await invoke<string>('load_template', { templateId });
            const parsed = JSON.parse(graphJson);
            const nodeCount = parsed.nodes?.length ?? 0;
            const workflow = await createWorkflow({
                name: `${templateId.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase())}`,
                description: `Created from template (${nodeCount} nodes)`,
                graphJson,
            });
            setSelectedWorkflow(workflow);
            addToast('Workflow created from template', 'success');
        } catch {
            addToast('Failed to load template', 'error');
        }
    };

    const handleBack = () => {
        setSelectedWorkflow(null);
    };

    if (selectedWorkflow) {
        return <WorkflowCanvas workflow={selectedWorkflow} onBack={handleBack} />;
    }

    return <WorkflowList onSelect={handleSelectWorkflow} onCreate={handleCreate} onCreateFromTemplate={handleCreateFromTemplate} />;
}
