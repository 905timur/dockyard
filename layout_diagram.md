graph TD
    A[Root Layout] --> B[Horizontal Split]
    B --> C[Left Pane: 20-25%]
    B --> D[Right Pane: 75-80%]
    D --> E[Vertical Split]
    E --> F[Top-Right Pane: 50%]
    E --> G[Bottom-Right Pane: 50%]

    C --> H[Container Details]
    F --> I[Container List]
    G --> J[Logs Last 10]