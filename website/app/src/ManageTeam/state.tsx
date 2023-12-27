import React, { createContext, useContext } from 'react';

// Define the context
interface IReadOnlyContextType {
    isReadOnly: boolean;
}

const ReadOnlyContext = createContext<IReadOnlyContextType | undefined>(undefined);

// Custom hook to access the read-only context
export function useReadOnly(): boolean {
    const context = useContext(ReadOnlyContext);
    if (!context) {
        throw new Error('useReadOnly must be used within a ReadOnlyProvider');
    }
    return context.isReadOnly;
}

// Provider component to wrap your app with
interface ReadOnlyProviderProps {
    children: React.ReactNode;
    isReadOnly: boolean;
}

export function ReadOnlyProvider({ children, isReadOnly }: ReadOnlyProviderProps): JSX.Element {
    return (
        <ReadOnlyContext.Provider value={{ isReadOnly }}>
            {children}
        </ReadOnlyContext.Provider>
    );
}
