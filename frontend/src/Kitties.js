import React, { useEffect, useState } from 'react';
import { Form, Grid } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

import KittyCards from './KittyCards';

export default function Kitties (props) {
  const { api, keyring } = useSubstrate();
  const { accountPair } = props;

  const [kittyCount, setKittyCnt] = useState(0);
  const [kittyDNAs, setKittyDNAs] = useState([]);
  const [kittyOwners, setKittyOwners] = useState([]);
  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('');

  const fetchKittyCnt = () => {
    api.query.kittiesModule.kittiesCount(count => {
      const countNum = count.toNumber();
      setKittyCnt(countNum);
    });
  };

  const fetchKitties = () => {
    let unSubDNAs = null;
    let unSubOwners = null;

    const asyncFetch = async () => {
      const kittyIndices = [...Array(kittyCount).keys()];
      unSubDNAs = await api.query.kittiesModule.kittyDB.multi(
        kittyIndices,
        dnas => setKittyDNAs(dnas.map(dna => dna.value.toU8a()))
      );
      unSubOwners = await api.query.kittiesModule.kittyOwner.multi(
        kittyIndices,
        owners => setKittyOwners(owners.map(owner => owner.toHuman()))
      );
    };
    
    asyncFetch();

    return () => {
      unSubDNAs && unSubDNAs();
      unSubOwners && unSubOwners();
    };
  };

  const populateKitties = () => {
    const kittyIndices = [...Array(kittyCount).keys()];
    const kitties = kittyIndices.map(n => ({
      id: n,
      dna: kittyDNAs[n],
      owner: kittyOwners[n],
    }));
    setKitties(kitties);
  };

  useEffect(fetchKittyCnt, [api, keyring]);
  useEffect(fetchKitties, [api, kittyCount]);
  useEffect(populateKitties, [kittyDNAs, kittyOwners]);

  return <Grid.Column width={16}>
    <h1>小毛孩</h1>
    <KittyCards kitties={kitties} accountPair={accountPair} setStatus={setStatus}/>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          accountPair={accountPair} label='创建小毛孩' type='SIGNED-TX' setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>;
}
