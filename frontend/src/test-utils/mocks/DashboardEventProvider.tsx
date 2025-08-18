import React, { createContext, useContext, ReactNode } from 'react';
import { DashboardEvent } from '../../types/dashboard';

interface DashboardEventContextType {
  emitEvent: (event: DashboardEvent) => void;
  addEventListener: (type: string, handler: (event: DashboardEvent) => void) => void;
  removeEventListener: (type: string, handler: (event: DashboardEvent) => void) => void;
}

const DashboardEventContext = createContext<DashboardEventContextType | undefined>(undefined);

interface DashboardEventProviderProps {
  children: ReactNode;
}

export const DashboardEventProvider: React.FC<DashboardEventProviderProps> = ({ children }) => {
  const eventHandlers = new Map<string, Set<(event: DashboardEvent) => void>>();

  const emitEvent = (event: DashboardEvent) => {
    const handlers = eventHandlers.get(event.type);
    if (handlers) {
      handlers.forEach(handler => handler(event));
    }
  };

  const addEventListener = (type: string, handler: (event: DashboardEvent) => void) => {
    if (!eventHandlers.has(type)) {
      eventHandlers.set(type, new Set());
    }
    eventHandlers.get(type)!.add(handler);
  };

  const removeEventListener = (type: string, handler: (event: DashboardEvent) => void) => {
    const handlers = eventHandlers.get(type);
    if (handlers) {
      handlers.delete(handler);
    }
  };

  return (
    <DashboardEventContext.Provider 
      value={{ emitEvent, addEventListener, removeEventListener }}
    >
      {children}
    </DashboardEventContext.Provider>
  );
};

export const useDashboardEvents = (): DashboardEventContextType => {
  const context = useContext(DashboardEventContext);
  if (!context) {
    throw new Error('useDashboardEvents must be used within a DashboardEventProvider');
  }
  return context;
};