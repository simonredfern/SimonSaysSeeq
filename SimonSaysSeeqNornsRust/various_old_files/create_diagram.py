#!/usr/bin/env python3
"""
Create a comprehensive diagram showing the SimonSaysSeeq sequencer user interface
and functions from a user's perspective.
"""

import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch, Rectangle, Circle
import numpy as np

def create_sequencer_diagram():
    # Create figure with high DPI for crisp PNG output
    fig, ax = plt.subplots(1, 1, figsize=(16, 12))
    ax.set_xlim(0, 16)
    ax.set_ylim(0, 12)
    ax.set_aspect('equal')
    ax.axis('off')
    
    # Title
    ax.text(8, 11.5, 'SimonSaysSeeq Rust Sequencer', 
            fontsize=24, fontweight='bold', ha='center')
    ax.text(8, 11, 'User Interface & Functions Overview', 
            fontsize=16, ha='center', style='italic')
    
    # Main Grid (Monome 16x8) - Primary Interface
    grid_x, grid_y = 1, 7.5
    grid_w, grid_h = 8, 4
    
    # Draw grid background
    grid_bg = FancyBboxPatch((grid_x, grid_y), grid_w, grid_h,
                             boxstyle="round,pad=0.1", 
                             facecolor='lightgray', 
                             edgecolor='black', linewidth=2)
    ax.add_patch(grid_bg)
    
    # Grid title
    ax.text(grid_x + grid_w/2, grid_y + grid_h + 0.3, 
            'Monome Grid (16×8)', 
            fontsize=14, fontweight='bold', ha='center')
    
    # Draw individual grid buttons
    button_size = 0.4
    spacing = 0.5
    
    # Sequence rows (0-6)
    for row in range(7):
        for col in range(16):
            x = grid_x + 0.3 + col * spacing
            y = grid_y + grid_h - 0.5 - row * spacing
            
            # Color active steps differently
            if (row + col) % 3 == 0:  # Simulate some active steps
                color = '#ff6b6b'  # Active step
                brightness = 0.8
            else:
                color = '#2c3e50'  # Inactive step
                brightness = 0.3
                
            button = Circle((x, y), button_size/2, 
                           facecolor=color, alpha=brightness,
                           edgecolor='white', linewidth=0.5)
            ax.add_patch(button)
    
    # Control row (row 7) - special functions
    for col in range(16):
        x = grid_x + 0.3 + col * spacing
        y = grid_y + grid_h - 0.5 - 7 * spacing
        
        # Highlight ARM function columns
        if col in [0, 1, 4, 5, 6, 7, 10]:
            color = '#f39c12'  # ARM functions
            brightness = 0.9
        else:
            color = '#34495e'
            brightness = 0.4
            
        button = Circle((x, y), button_size/2, 
                       facecolor=color, alpha=brightness,
                       edgecolor='white', linewidth=0.5)
        ax.add_patch(button)
    
    # Grid function labels
    ax.text(grid_x - 0.5, grid_y + grid_h - 1, 'Rows 0-6:', 
            fontsize=10, fontweight='bold', rotation=90, va='center')
    ax.text(grid_x - 0.7, grid_y + grid_h - 1.5, 'Sequence\nSteps', 
            fontsize=9, rotation=90, va='center')
    
    ax.text(grid_x - 0.5, grid_y + 0.5, 'Row 7:', 
            fontsize=10, fontweight='bold', rotation=90, va='center')
    ax.text(grid_x - 0.7, grid_y + 0.3, 'Control/ARM\nFunctions', 
            fontsize=9, rotation=90, va='center')
    
    # Hardware Controls
    hw_x, hw_y = 10.5, 8.5
    hw_w, hw_h = 4.5, 3
    
    hw_bg = FancyBboxPatch((hw_x, hw_y), hw_w, hw_h,
                           boxstyle="round,pad=0.1",
                           facecolor='#ecf0f1',
                           edgecolor='black', linewidth=2)
    ax.add_patch(hw_bg)
    
    ax.text(hw_x + hw_w/2, hw_y + hw_h + 0.2, 
            'Hardware Controls', 
            fontsize=14, fontweight='bold', ha='center')
    
    # Encoders
    encoder_y = hw_y + hw_h - 0.8
    for i, (label, func) in enumerate([('E1', 'Tempo'), ('E2', 'Swing'), ('E3', 'Transpose')]):
        x = hw_x + 0.7 + i * 1.2
        
        # Draw encoder
        encoder = Circle((x, encoder_y), 0.3, 
                        facecolor='#3498db', 
                        edgecolor='black', linewidth=1)
        ax.add_patch(encoder)
        
        # Encoder labels
        ax.text(x, encoder_y, label, fontsize=10, fontweight='bold', 
                ha='center', va='center', color='white')
        ax.text(x, encoder_y - 0.6, func, fontsize=9, 
                ha='center', va='center')
    
    # Keys
    key_y = hw_y + 0.8
    for i, (label, func) in enumerate([('K1', 'Undo'), ('K2', 'Stop'), ('K3', 'Start/Stop')]):
        x = hw_x + 0.7 + i * 1.2
        
        # Draw key
        key = Rectangle((x-0.2, key_y-0.15), 0.4, 0.3,
                       facecolor='#e74c3c',
                       edgecolor='black', linewidth=1)
        ax.add_patch(key)
        
        # Key labels
        ax.text(x, key_y, label, fontsize=10, fontweight='bold', 
                ha='center', va='center', color='white')
        ax.text(x, key_y - 0.5, func, fontsize=9, 
                ha='center', va='center')
    
    # Framework RGB Macropad
    macro_x, macro_y = 10.5, 5
    macro_w, macro_h = 2.5, 2.5
    
    macro_bg = FancyBboxPatch((macro_x, macro_y), macro_w, macro_h,
                              boxstyle="round,pad=0.1",
                              facecolor='#2c3e50',
                              edgecolor='black', linewidth=2)
    ax.add_patch(macro_bg)
    
    ax.text(macro_x + macro_w/2, macro_y + macro_h + 0.2, 
            'RGB Macropad (4×4)', 
            fontsize=12, fontweight='bold', ha='center')
    
    # Draw 4x4 macropad buttons
    for row in range(4):
        for col in range(4):
            x = macro_x + 0.3 + col * 0.5
            y = macro_y + macro_h - 0.4 - row * 0.5
            
            # Rainbow colors for visual appeal
            colors = ['#ff0000', '#ff7f00', '#ffff00', '#00ff00',
                     '#0000ff', '#4b0082', '#9400d3', '#ff1493',
                     '#00ffff', '#ff69b4', '#adff2f', '#ffa500',
                     '#da70d6', '#98fb98', '#f0e68c', '#dda0dd']
            color = colors[row * 4 + col]
            
            button = Rectangle((x-0.15, y-0.15), 0.3, 0.3,
                              facecolor=color, alpha=0.8,
                              edgecolor='white', linewidth=1)
            ax.add_patch(button)
    
    # Display & Feedback
    display_x, display_y = 1, 5
    display_w, display_h = 8, 2
    
    display_bg = FancyBboxPatch((display_x, display_y), display_w, display_h,
                                boxstyle="round,pad=0.1",
                                facecolor='#34495e',
                                edgecolor='black', linewidth=2)
    ax.add_patch(display_bg)
    
    ax.text(display_x + display_w/2, display_y + display_h + 0.2, 
            'OLED Display (128×64)', 
            fontsize=14, fontweight='bold', ha='center')
    
    # Simulate display content
    ax.text(display_x + display_w/2, display_y + display_h - 0.3, 
            '♪ RUNNING | 120.0 BPM | Step: 5/16 | Bar: 1', 
            fontsize=12, fontweight='bold', ha='center', color='white')
    ax.text(display_x + display_w/2, display_y + display_h - 0.8, 
            'Pattern: A | Swing: 10% | Transpose: +2', 
            fontsize=10, ha='center', color='#bdc3c7')
    ax.text(display_x + display_w/2, display_y + display_h - 1.3, 
            'MIDI: Ch1 Out | Grid: Connected', 
            fontsize=10, ha='center', color='#95a5a6')
    
    # ARM Functions Legend
    arm_x, arm_y = 1, 2.5
    arm_w, arm_h = 6, 2
    
    arm_bg = FancyBboxPatch((arm_x, arm_y), arm_w, arm_h,
                           boxstyle="round,pad=0.1",
                           facecolor='#fff3cd',
                           edgecolor='#856404', linewidth=2)
    ax.add_patch(arm_bg)
    
    ax.text(arm_x + arm_w/2, arm_y + arm_h + 0.2, 
            'ARM Functions (Row 7)', 
            fontsize=14, fontweight='bold', ha='center')
    
    # ARM function list
    arm_functions = [
        'Col 0: Undo', 'Col 1: Redo', 'Col 4: Euclidean Events',
        'Col 5: Euclidean Length', 'Col 6: Euclidean Rotation',
        'Col 7: Ratchet', 'Col 10: Preset Grid'
    ]
    
    for i, func in enumerate(arm_functions):
        x = arm_x + 0.2 + (i % 2) * 3
        y = arm_y + arm_h - 0.4 - (i // 2) * 0.35
        ax.text(x, y, f"• {func}", fontsize=9, va='center')
    
    # Features & Capabilities
    features_x, features_y = 8, 2.5
    features_w, features_h = 7, 2
    
    features_bg = FancyBboxPatch((features_x, features_y), features_w, features_h,
                                boxstyle="round,pad=0.1",
                                facecolor='#d1ecf1',
                                edgecolor='#0c5460', linewidth=2)
    ax.add_patch(features_bg)
    
    ax.text(features_x + features_w/2, features_y + features_h + 0.2, 
            'Key Features & Capabilities', 
            fontsize=14, fontweight='bold', ha='center')
    
    features = [
        '• High-performance Rust sequencing engine',
        '• 16-step × 7-track pattern sequencing',
        '• Real-time MIDI I/O with note tracking',
        '• Pattern chains & management',
        '• Euclidean rhythm generation',
        '• Swing timing & global transpose',
        '• Environmental data integration (CO2)',
        '• RGB LED feedback & animations',
        '• Undo/Redo with full state snapshots',
        '• Simulation mode for development'
    ]
    
    for i, feature in enumerate(features):
        x = features_x + 0.2 + (i % 2) * 3.3
        y = features_y + features_h - 0.3 - (i // 2) * 0.25
        ax.text(x, y, feature, fontsize=8, va='center')
    
    # Workflow arrows and connections
    # Arrow from grid to display
    ax.annotate('', xy=(display_x + display_w/2, display_y + display_h), 
                xytext=(grid_x + grid_w/2, grid_y),
                arrowprops=dict(arrowstyle='->', lw=2, color='#e74c3c'))
    
    # Arrow from hardware to display  
    ax.annotate('', xy=(display_x + display_w, display_y + display_h/2), 
                xytext=(hw_x, hw_y + hw_h/2),
                arrowprops=dict(arrowstyle='->', lw=2, color='#3498db'))
    
    # Mode indicators
    ax.text(0.2, 0.8, 'Modes:', fontsize=12, fontweight='bold')
    ax.text(0.2, 0.4, '• Hardware Mode (Full Norns integration)', fontsize=10)
    ax.text(0.2, 0.1, '• Simulation Mode (Keyboard testing)', fontsize=10)
    
    # Save as high-quality PNG
    plt.tight_layout()
    plt.savefig('sequencer_ui_diagram.png', 
                dpi=300, bbox_inches='tight', 
                facecolor='white', edgecolor='none')
    plt.close()
    
    print("✅ Sequencer UI diagram created: sequencer_ui_diagram.png")

if __name__ == "__main__":
    create_sequencer_diagram()