import { createTheme, MantineProvider } from "@mantine/core";

import "./style/index.scss";
import { Layout } from "./views/Layout";

export function App() {
    return (
        <MantineProvider
            theme={createTheme({
                colors: {
                    primary: [
                        "#ffeaf3",
                        "#fdd4e1",
                        "#f4a7bf",
                        "#ec779c",
                        "#e64f7e",
                        "#e3356b",
                        "#e22762",
                        "#c91a52",
                        "#b41149",
                        "#9f003e",
                    ],
                    dark: [
                        "#C9C9C9",
                        "#696969",
                        "#424242",
                        "#3b3b3b",
                        "#2e2e2e",
                        "#242424",
                        "#1f1f1f",
                        "#121212",
                        "#101010",
                        "#0a0a0a",
                    ],
                },
                primaryColor: "primary",
                primaryShade: 7,
                autoContrast: true,
            })}
            defaultColorScheme="dark"
        >
            <Layout />
        </MantineProvider>
    );
}
