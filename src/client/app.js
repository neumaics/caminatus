import React from 'react';
import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';
import styled from 'styled-components';

import { Schedules } from './schedules';

const StyledTabs = styled(Tabs)`
  display: grid;
  grid-template-columns: 140px auto;
  grid-template-rows: 100%;
  height: 100%;

`;

const StyledTabList = styled(TabList)`
  background-color: #2C3033;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  list-style-type: none;
  width: 140px;
  height: 100%;

  padding: 0;
  margin: 0;
  
  grid-column: 1 / 1;
`;

const StyledTabPanelContainer = styled.div`
  grid-column-start: 2 / 2;
`;

// TODO: use theme to pull colors...?
const StyledTab = styled(Tab)`
  padding: 16px;
  background-color: ${props => props.selected ? '#0A0F12' : '#2C3033'};
  font-size: 18px;
  width: 140px;
  text-align: right;
`;

export const App = () => (
  <StyledTabs>
    <StyledTabList>
      <StyledTab>Current</StyledTab>
      <StyledTab>Schedules</StyledTab>
      <StyledTab>Settings</StyledTab>
    </StyledTabList>

    <StyledTabPanelContainer>
      <TabPanel>
        <div>Current</div>
      </TabPanel>
      <TabPanel>
        <Schedules />
      </TabPanel>
      <TabPanel>
        <div>Settings</div>
      </TabPanel>
      </StyledTabPanelContainer>
  </StyledTabs>
);
