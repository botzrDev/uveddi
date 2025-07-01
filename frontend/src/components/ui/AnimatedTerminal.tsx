import React, { useEffect, useState } from 'react';

interface TerminalCommand {
  command: string;
  output: string[];
  delay?: number;
}

interface AnimatedTerminalProps {
  commands: TerminalCommand[];
  className?: string;
  autoStart?: boolean;
  loop?: boolean;
}

const AnimatedTerminal: React.FC<AnimatedTerminalProps> = ({
  commands,
  className = '',
  autoStart = true,
  loop = false,
}) => {
  const [currentCommandIndex, setCurrentCommandIndex] = useState(0);
  const [currentTyping, setCurrentTyping] = useState('');
  const [showOutput, setShowOutput] = useState(false);
  const [isComplete, setIsComplete] = useState(false);

  useEffect(() => {
    if (!autoStart || isComplete) return;

    const currentCommand = commands[currentCommandIndex];
    if (!currentCommand) return;

    let charIndex = 0;
    const typeCommand = () => {
      if (charIndex < currentCommand.command.length) {
        setCurrentTyping(currentCommand.command.slice(0, charIndex + 1));
        charIndex++;
        setTimeout(typeCommand, 50 + Math.random() * 50);
      } else {
        setTimeout(() => {
          setShowOutput(true);
          setTimeout(() => {
            if (currentCommandIndex < commands.length - 1) {
              setCurrentCommandIndex(currentCommandIndex + 1);
              setCurrentTyping('');
              setShowOutput(false);
            } else {
              setIsComplete(true);
              if (loop) {
                setTimeout(() => {
                  setCurrentCommandIndex(0);
                  setCurrentTyping('');
                  setShowOutput(false);
                  setIsComplete(false);
                }, 2000);
              }
            }
          }, currentCommand.delay || 1500);
        }, 300);
      }
    };

    const timer = setTimeout(typeCommand, 500);
    return () => clearTimeout(timer);
  }, [currentCommandIndex, autoStart, isComplete, commands, loop]);

  const currentCommand = commands[currentCommandIndex];

  return (
    <div className={`bg-secondary-900 border border-secondary-700 rounded-lg p-6 font-mono text-sm ${className}`}>
      <div className="flex items-center mb-4">
        <div className="flex space-x-2">
          <div className="w-3 h-3 bg-red-500 rounded-full"></div>
          <div className="w-3 h-3 bg-accent-400 rounded-full"></div>
          <div className="w-3 h-3 bg-primary-500 rounded-full"></div>
        </div>
        <span className="ml-4 text-secondary-400 text-xs">uveddi-terminal</span>
      </div>
      
      <div className="space-y-2">
        {commands.slice(0, currentCommandIndex).map((cmd, index) => (
          <div key={index}>
            <div className="flex items-center">
              <span className="text-primary-400 mr-2">$</span>
              <span className="text-secondary-100">{cmd.command}</span>
            </div>
            {cmd.output.map((line, lineIndex) => (
              <div key={lineIndex} className="text-secondary-300 ml-4">
                {line}
              </div>
            ))}
          </div>
        ))}
        
        {currentCommand && (
          <div>
            <div className="flex items-center">
              <span className="text-primary-400 mr-2">$</span>
              <span className="text-secondary-100">{currentTyping}</span>
              <span className="animate-terminal-cursor text-primary-400">|</span>
            </div>
            {showOutput && currentCommand.output.map((line, index) => (
              <div key={index} className="text-secondary-300 ml-4 animate-fade-in">
                {line}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default AnimatedTerminal;
