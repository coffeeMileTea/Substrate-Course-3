import React from 'react';
import { Button, Card, Grid, Message, Modal, Form, Label } from 'semantic-ui-react';

import KittyAvatar from './KittyAvatar';
import { TxButton } from './substrate-lib/components';
import { useSubstrate } from './substrate-lib';

// --- About Modal ---

const TransferModal = props => {
  const { kitty, accountPair, setStatus } = props;
  const [open, setOpen] = React.useState(false);
  const [formValue, setFormValue] = React.useState({});

  const formChange = key => (ev, el) => {
    setFormValue({ ...formValue, [key]: el.value });
  };

  const confirmAndClose = (unsub) => {
    unsub();
    setOpen(false);
  };

  return <Modal onClose={() => setOpen(false)} onOpen={() => setOpen(true)} open={open}
    trigger={<Button basic color='yellow'>转让</Button>}>
    <Modal.Header>毛孩转让</Modal.Header>
    <Modal.Content><Form>
      <Form.Input fluid label='毛孩 ID' readOnly value={kitty.id}/>
      <Form.Input fluid label='转让对象' placeholder='对方地址' onChange={formChange('target')}/>
    </Form></Modal.Content>
    <Modal.Actions>
      <Button basic color='grey' onClick={() => setOpen(false)}>取消</Button>
      <TxButton
        accountPair={accountPair} label='确认转让' type='SIGNED-TX' setStatus={setStatus}
        onClick={confirmAndClose}
        attrs={{
          palletRpc: 'kittiesModule',
          callable: 'transfer',
          inputParams: [formValue.target, kitty.id],
          paramFields: [true, true]
        }}
      />
    </Modal.Actions>
  </Modal>;
};

// --- About Kitty Card ---

const KittyCard = props => {
  const { keyring } = useSubstrate();
  const accounts = keyring.getPairs();
  
  const { kitty, accountPair, setStatus } = props;
  const { id = null, dna = null, owner = null, price = null } = kitty;
  const displayDna = dna && dna.join(', ');
  const displayPrice = price || '不出售';
  const displayId = id === null ? '' : id
  const isOwned = accountPair.address === kitty.owner;
  let kitty_owner_name = "unknow";
  accounts.map(account =>{
    if (account.address === kitty.owner) {
      kitty_owner_name = account.meta.name
    }
  })

  return <Card>
    { isOwned && <Label as='a' floating color='teal'>我的</Label> }
    <KittyAvatar dna={dna} />
    <Card.Content>
      <Card.Header>ID 号: {displayId}</Card.Header>
      <Card.Meta style={{ overflowWrap: 'break-word' }}>
        基因: <br/>
        {displayDna}
      </Card.Meta>
      <Card.Description>
        <p style={{ overflowWrap: 'break-word' }}>
          主人: <br/>
          {kitty_owner_name} ({owner})
        </p>
        <p>{displayPrice}</p>
      </Card.Description>
    </Card.Content>
    <Card.Content extra style={{ textAlign: 'center' }}>{ owner === accountPair.address
      ? <TransferModal kitty={kitty} setStatus={setStatus} accountPair={accountPair}/>
      : ''
    }</Card.Content>
  </Card>;
};

const KittyCards = props => {
  const { kitties, accountPair, setStatus } = props;

  if (kitties.length === 0) {
    return <Message info>
      <Message.Header>现在连一只毛孩都木有，赶快创建一只&nbsp;
        <span role='img' aria-label='point-down'>👇</span>
      </Message.Header>
    </Message>;
  }

  return <Grid columns={3}>{kitties.map((kitty, i) =>
    <Grid.Column key={`kitty-${i}`}>
      <KittyCard kitty={kitty} setStatus={setStatus} accountPair={accountPair}/>
    </Grid.Column>
  )}</Grid>;
};

export default KittyCards;
