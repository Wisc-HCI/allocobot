import React from 'react';
import ReactDOM from 'react-dom/client';
import {
  createBrowserRouter,
  // LoaderFunctionArgs,
  RouterProvider,
} from "react-router-dom";
import Root from './routes/root.tsx';
import ErrorPage from "./error-page";
import './index.css';
import Place from './routes/place.tsx';
import Transition from './routes/transition.tsx';

// type PlaceArgs = {
//   placeId: string;
// }

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root/>,
    errorElement: <ErrorPage />,
    children: [
      {
        path: "places/:placeId",
        element: <Place/>,
      },
      {
        path: "transitions/:transitionId",
        element: <Transition/>,
      },
    ],
  }
]);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
)
