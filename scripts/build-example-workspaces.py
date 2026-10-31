#!/usr/bin/env python3
"""Author the bundled examples through the real local MCP server.

Build the CLI first. Pass an empty directory OUTSIDE the checkout; inspect the
results before copying the two JSON documents into workspaces/. No direct JSON
workspace writes are used. Every mutation is followed by a persisted lens read.
"""
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


class Canvas:
    def __init__(self, directory, slug, title):
        self.path = directory / (slug + '.infinite-note.json')
        if self.path.exists():
            raise ValueError(f'Refusing to replace {self.path}')
        self.process = subprocess.Popen(
            [str(ROOT / 'target/release/infinite-note-canvas-cli'), 'mcp',
             '--workspace', str(self.path), '--allow-write'],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        self.sequence = 0
        self.writes = 0
        self.ids = {}
        self.call('initialize', {'protocolVersion': '2025-06-18',
                  'capabilities': {}, 'clientInfo': {'name': 'reason-room-examples', 'version': '1'}})
        self.call('tools/list', {})
        self.tool('create_workspace', {'title': title, 'workspace_id': slug})
        self.call('tools/list', {})
        self.tool('workspace_overview', {})
        self.read()
        self.layers = []
        for name, start, end in [('Overview — zoom into a stage', 1, 24),
                                  ('Process — follow the flow', 24, 85),
                                  ('Detail — inspect the work', 85, 409600)]:
            self.change('create_zoom_layer', title=name, start_percent=start,
                        end_percent=end, fade_percent=2)
            layer = self.lens['semantic_zoom_layers'][-1]
            self.layers.append(layer['id'])
            self.change('update_zoom_layer', layer_id=layer['id'], title=name,
                        start_percent=start, end_percent=end, fade_percent=2, card_scale=1)

    def call(self, method, params):
        self.sequence += 1
        message = {'jsonrpc': '2.0', 'id': self.sequence, 'method': method, 'params': params}
        self.process.stdin.write(json.dumps(message) + '\n')
        self.process.stdin.flush()
        result = json.loads(self.process.stdout.readline())
        if 'error' in result:
            raise RuntimeError(result)
        return result['result']

    def tool(self, name, arguments):
        result = self.call('tools/call', {'name': name, 'arguments': arguments})
        if result.get('isError'):
            raise RuntimeError(result)
        return result.get('structuredContent') or json.loads(result['content'][0]['text'])

    def read(self):
        self.lens = self.tool('read_lens', {'limit': 250})
        return self.lens

    def change(self, kind, **fields):
        result = self.tool('apply_workspace_change', {'change': {'kind': kind, **fields}})
        assert result['workspace_digest']
        self.writes += 1
        self.read()
        return result

    def card(self, key, title, kind, bounds, level, summary, details='', attributes=None):
        result = self.change('create_card', type_id=kind, title=title,
                             bounds=bounds, summary=summary)
        card_id = result['selected_concept_ids'][0]
        self.ids[key] = card_id
        self.change('set_card_layer_mode', concept_id=card_id,
                    mode={'mode': 'manual', 'layer_ids': self.layers[level:]})
        self.change('update_card', concept_id=card_id, details=details or summary,
                    attributes=attributes or {})
        return card_id

    def edge(self, source, target, kind='delivers'):
        self.change('create_relationship', source_id=self.ids[source],
                    target_id=self.ids[target], relationship_kind=kind)

    def stage(self, key, index, title, kind, summary, steps):
        col, row = index % 3, index // 3
        x, y = col * 2200, row * 1650
        self.card(key, title, kind, box(x, y, 1800, 1250), 0, summary,
                  summary + '\n\nZoom into this card: process cards appear at 24%; individual work records at 85%. '
                  'This card is a spatial container. Its placement does not imply ownership or a typed contains edge.')
        for j, (name, text, leaves) in enumerate(steps):
            child = f'{key}.{j}'
            self.card(child, name, 'process', box(x + 60 + j * 570, y + 300, 510, 870),
                      1, text, attributes={'example': 'Illustrative; not production instructions'})
            for k, (leaf, note) in enumerate(leaves):
                self.card(f'{child}.{k}', leaf, 'service',
                          box(x + 85 + j * 570, y + 570 + k * 260, 460, 210), 2, note)
            if len(leaves) == 2:
                self.edge(child + '.0', child + '.1')
        for j in range(len(steps) - 1):
            self.edge(f'{key}.{j}', f'{key}.{j+1}')
        return (x, y)

    def boundary(self, title, x, y, width, height, summary):
        self.change('create_boundary', title=title, bounds=box(x, y, width, height), summary=summary)

    def finish(self):
        counts = []
        for scale in [.12, .45, 1.2]:
            frame = self.tool('preview_semantic_zoom', {'scale': scale})
            counts.append(len(frame['cards']))
        assert counts[0] < counts[1] < counts[2], counts
        self.tool('query_workspace', {'concept_id': next(iter(self.ids.values())),
                                     'direction': 'both', 'depth': 2})
        overview = self.tool('workspace_overview', {})
        self.process.stdin.close()
        assert self.process.wait(timeout=15) == 0
        print(json.dumps({'workspace': self.path.name, 'mutations': self.writes,
                          'visible_cards_at_12_45_120_percent': counts,
                          'digest': overview['workspace_digest']}))


def box(x, y, width, height):
    return {'origin': {'x': x, 'y': y}, 'size': {'x': width, 'y': height}}


def step(name, summary, first, second):
    return (name, summary, [first, second])


def website(directory):
    c = Canvas(directory, 'static-react-website', 'Static React website — source to visitor')
    stages = [
        ('repo', 'GitHub repository', 'repository', 'Start here: source and documentation travel through CI to a static website. Zoom into each large card. This is an illustrative architecture, not a live cloud connection.', [
            step('src/', 'React source lives in the repository.', ('App.tsx + components/', 'Page composition and reusable UI; no server runtime is assumed.'), ('assets/ + styles/', 'Versioned images and CSS become static build assets.')),
            step('docs/ + README', 'Versioned documentation ships with source.', ('README.md', 'Setup, local development and links to the wiki.'), ('docs/architecture.md', 'Document build output, DNS path and release responsibilities.')),
            step('GitHub wiki', 'Separate Git repository attached to the GitHub project; not a folder in src/.', ('Operations pages', 'Release checklist, rollback and ownership notes.'), ('Contributor guide', 'Link back to README and source documentation to avoid drift.'))]),
        ('actions', 'GitHub Actions', 'process', 'A push to master triggers .github/workflows/deploy.yml to build and publish. The workflow belongs to the repository; short-lived AWS credentials use OIDC.', [
            step('Checkout + install', 'A runner checks out the commit and installs locked dependencies.', ('actions/checkout', 'Read the exact commit associated with the push.'), ('npm ci', 'Use package-lock.json; test before release.')),
            step('Test + build', 'A static React build produces dist/.', ('npm test', 'Example validation step; exact test command depends on the project.'), ('npm run build', 'Emit dist/index.html and hashed static assets.')),
            step('Assume role + publish', 'GitHub OIDC exchanges a scoped identity for temporary AWS credentials.', ('OIDC trust policy', 'Restrict repository, branch and audience. No long-lived access key in source.'), ('Sync dist/ to S3', 'Upload static files, set cache policy and invalidate changed CDN paths.'))]),
        ('aws', 'AWS static hosting', 'environment', 'A private S3 origin stores the website. CloudFront provides HTTPS for the public hostname. Cloudflare manages DNS only in this example.', [
            step('IAM release role', 'The pipeline may publish only to the intended bucket and distribution.', ('Trust GitHub OIDC', 'Trust policy limits which workflow identity can assume this role.'), ('Publish permissions', 'Scope S3 object changes and CDN invalidation to this site.')),
            step('S3 bucket', 'Static origin, private to CloudFront; not a backend application.', ('index.html', 'Short cache lifetime for the entry document.'), ('assets/*.js + *.css', 'Immutable hashed files may use long cache lifetimes.')),
            step('CloudFront + TLS', 'HTTPS terminates at CloudFront, which reads S3 through origin access control.', ('Origin access control', 'Keep the bucket private; authorize this distribution.'), ('ACM certificate', 'Certificate covers www.example.com; SPA route fallback configured at the CDN.'))]),
        ('dns', 'Cloudflare DNS', 'environment', 'An independent Cloudflare account owns the DNS zone. The hostname resolves to CloudFront, not directly to a private bucket.', [
            step('Cloudflare account', 'Administrative boundary separate from GitHub and AWS.', ('Zone access', 'Restrict who can change records.'), ('example.com zone', 'Illustrative domain; no account or live domain is configured.')),
            step('Public DNS records', 'DNS-only record in this example; no additional Cloudflare proxy/cache layer.', ('www CNAME', 'www.example.com points to the CloudFront distribution hostname.'), ('TLS validation record', 'Publish the certificate validation CNAME requested by ACM.')),
            step('Resolution', 'Resolver follows authoritative answers to the CDN.', ('DNS lookup', 'The browser resolves www.example.com.'), ('CDN address', 'DNS returns an endpoint; it does not upload site files.'))]),
        ('visitor', 'Website visitors', 'organization', 'The request path crosses DNS and AWS boundaries. No visitor request goes to GitHub Actions.', [
            step('Find the website', 'A person opens the public hostname.', ('Enter URL', 'https://www.example.com is illustrative.'), ('Resolve hostname', 'Recursive resolver consults authoritative Cloudflare DNS.')),
            step('Fetch static files', 'HTTPS request goes to CloudFront.', ('GET /', 'CDN returns index.html, reading its S3 origin on cache miss.'), ('GET /assets/*', 'Browser fetches JS, CSS and images with cacheable URLs.')),
            step('Render React', 'React executes in the visitor browser.', ('Mount application', 'App.tsx renders the interface.'), ('Client navigation', 'SPA routes need an entry-document fallback on direct navigation.'))]),
        ('release', 'Release & recovery', 'process', 'Trace a versioned commit to its deployed files. Knowledge and recovery paths cross repository, wiki and AWS boundaries.', [
            step('Review release', 'A maintainer approves a source change.', ('Pull request', 'Review source, documentation and workflow changes together.'), ('Merge to master', 'Triggers the example release workflow.')),
            step('Verify publication', 'Check the served artifact, not just a green build.', ('Smoke check', 'Open the public URL and a direct SPA route.'), ('Record commit SHA', 'Relate deployed output to the GitHub revision.')),
            step('Recover', 'Use a previous known-good artifact or rebuild a selected revision.', ('Rollback artifact', 'Re-publish a known-good dist/ and invalidate stale entry documents.'), ('Wiki runbook', 'Record decision, validation and follow-up in the project wiki.'))])]
    for i, args in enumerate(stages):
        c.stage(args[0], i, *args[1:])
    for key, target, kind in [('actions','repo','implemented_in'), ('actions','aws','deployed_to'),
                              ('visitor','dns','depends_on'), ('visitor','aws','depends_on'),
                              ('dns','aws','depends_on'), ('release','repo','implemented_in')]:
        c.edge(key, target, kind)
    for a,b,k in [('repo.0','actions.0','delivers'), ('actions.2','aws.1','deployed_to'),
                  ('dns.1','aws.2','depends_on'), ('visitor.1','aws.2','depends_on'),
                  ('release.2','repo.2','depends_on'), ('repo.1','release.0','delivers')]:
        c.edge(a,b,k)
    for a,b,k in [('actions.1.1','aws.1.0','deployed_to'), ('dns.1.0','aws.2.1','depends_on'),
                  ('visitor.1.1','aws.1.1','depends_on'), ('release.2.1','repo.2.0','depends_on')]:
        c.edge(a,b,k)
    # Regions are independent of spatial card containment; deliberately cross
    # the reading grid so the trust boundaries remain visibly distinct.
    for title,x,y,w,h,note in [
        ('GitHub — code, automation & knowledge',-100,-170,4100,1520,'Source repository, workflow and wiki are different GitHub resources.'),
        ('AWS account — hosting',4300,-170,2000,1520,'Private S3 origin with public CloudFront HTTPS.'),
        ('Cloudflare account — DNS authority',-100,1480,2000,1520,'DNS administration is outside the AWS account.'),
        ('Public client',2100,1480,2000,1520,'Visitors cross DNS and hosting boundaries.'),
        ('Delivery operations',4300,1480,2000,1520,'Release and recovery spans multiple accounts.')]:
        c.boundary(title,x,y,w,h,note)
    c.finish()


def cars(directory):
    c = Canvas(directory, 'car-manufacturing-vsm', 'Car manufacturing — value stream from mine to customer')
    data = [
        ('mine','Mining & materials','organization','External supplier: raw ore becomes usable steel and aluminum. All figures and organizations in this training map are illustrative.',[
            step('Extract ore','Upstream extraction precedes the factory value stream.',('Drill + blast','Prepare and extract ore; material flow starts here.'),('Crush + concentrate','Separate valuable material before transport.')),
            step('Smelt + refine','Convert concentrate into primary metal.',('Smelt','Energy-intensive conversion to usable metal.'),('Refine + cast','Control composition and cast slabs or billets.')),
            step('Roll + dispatch','Produce coils that component suppliers can form.',('Roll steel coils','Record batch identity and grade.'),('Load rail shipment','A transport queue adds lead time without adding processing time.'))]),
        ('supplier','Component fabrication','organization','External tier suppliers convert materials into stamped panels, machined parts and bought-in assemblies.',[
            step('Form + machine','Supplier conversion work.',('Stamp panels','Press coil blanks into body panels; track scrap and changeover.'),('Machine powertrain','Machine housings and engine or drive-unit components.')),
            step('Build subassemblies','Parts become modules.',('Seats + electronics','Assemble interiors and electronic modules.'),('Tires + glass','Fabricate bought-in parts to the vehicle specification.')),
            step('Quality + pack','Supplier release gate.',('Inspect batch','Trace dimensions, lot and nonconformance.'),('Pack returnable racks','Protect parts and label destination and sequence.'))]),
        ('inbound','Inbound logistics','process','External transport crosses supplier and company boundaries. Waiting, transit and customs are lead time, not vehicle assembly cycle time.',[
            step('Schedule pickup','Respond to factory replenishment signals.',('Transport order','Consolidate supplier lots and delivery windows.'),('Milk-run route','Collect sequenced racks from multiple suppliers.')),
            step('Move freight','Material travels by rail and truck.',('Rail line-haul','Long-distance movement of coils or components.'),('Truck last mile','Deliver to the factory receiving dock.')),
            step('Dock appointment','Control arrival and unloading queues.',('Arrival queue','Illustrative wait: 0.5 day.'),('Unload + scan','Match rack identifiers to the advance shipment notice.'))]),
        ('warehouse','Receiving & warehouse','environment','Company boundary begins at receiving. Inventory triangles in a traditional VSM are represented here by explicit queue cards.',[
            step('Receive + inspect','Confirm quantity and supplier quality.',('Goods receipt','Record supplier lot and received quantity.'),('Incoming quality','Quarantine nonconforming material.')),
            step('Store inventory','Waiting ties up material and cash.',('Raw-material queue','Illustrative inventory: 960 car-equivalents = 2 days at 480 cars/day.'),('Warehouse locations','FIFO by lot; record dwell time and replenishment point.')),
            step('Feed production','Pull parts to the line.',('Kit + sequence','Prepare the next vehicle specification.'),('Line-side delivery','Tugger supplies point-of-use racks on a repeatable route.'))]),
        ('body','Stamping & body shop','process','Factory transformation: stamping feeds welding. C/T, changeover, uptime and queues are illustrative training values; zoom in to inspect.',[
            step('Stamping','C/T 60 s/car-equivalent; C/O 30 min; uptime 90%; 2 operators.',('Blank + press','Transfer sheet through dies; die changes drive batches.'),('Panel supermarket','Illustrative WIP 240 equivalents = 0.5 day at customer demand.')),
            step('Body welding','C/T 100 s/car; C/O 10 min; uptime 95%; 6 operators.',('Fixture + join','Locate panels and robot-weld the body-in-white.'),('Dimensional check','Measure critical points before paint.')),
            step('Body buffer','Protect paint from upstream variation.',('Body queue','Illustrative WIP 120 cars = 0.25 day.'),('Release to paint','Sequence bodies against color and production demand.'))]),
        ('paint','Paint shop','process','Illustrative bottleneck: effective C/T 125 s/car exceeds takt 112.5 s/car. Batch ovens add residence time beyond this equivalent cycle time.',[
            step('Pretreat + e-coat','Clean and protect the metal body.',('Clean + phosphate','Surface preparation controls coating adhesion.'),('E-coat dip','Apply corrosion protection and drain.')),
            step('Apply + cure','C/T 125 s/car equivalent; C/O 20 min; uptime 85%; 8 operators.',('Primer + color','Apply coatings; group colors to reduce changeovers.'),('Oven cure','Batch residence time is distinct from line output cadence.')),
            step('Inspect + buffer','Rework and queues can dominate lead time.',('Finish inspection','Record defects and route rework separately.'),('Painted-body queue','Illustrative WIP 480 cars = 1 day; investigate bottleneck impact.'))]),
        ('assembly','Final assembly & test','process','C/T 110 s/car; C/O 0 min model-mix assumption; uptime 95%; 24 operators. Compare each station with takt before inferring total capacity.',[
            step('Trim + chassis','Fit modules to the painted body.',('Interior trim','Install harness, cockpit, seats and glazing.'),('Chassis marriage','Join body with suspension and powertrain.')),
            step('Complete vehicle','Sequence parts by vehicle specification.',('Fluids + wheels','Fill systems, mount wheels and torque critical joints.'),('Software + calibration','Load configuration and calibrate vehicle systems.')),
            step('End-of-line test','Illustrative C/T 80 s/car; repair loop shown in notes.',('Safety + function','Brake, electrical and leak checks.'),('Release or repair','Quality gate releases conforming cars; defects return for repair and retest.'))]),
        ('outbound','Finished goods & delivery','environment','Company yard and external transport separate manufacturing completion from customer receipt.',[
            step('Finished-car warehouse','Inventory waits for dealer allocation.',('Vehicle yard','Illustrative inventory 720 cars = 1.5 days at demand.'),('Allocate VIN','Match specification and sales order.')),
            step('Outbound logistics','Cross the factory boundary to the dealer.',('Rail or car carrier','Illustrative transport + queue: 2 days.'),('Dealer receipt','Check VIN, condition and delivery record.')),
            step('Delivery confirmation','Close the material-flow record.',('Proof of delivery','Record custody transfer and damage exceptions.'),('Replenishment feedback','Actual sales inform production planning, not only dispatch counts.'))]),
        ('sales','Dealer lot & customer demand','organization','External downstream boundary. Demand 480 cars/day; available production time 2 × 450 min/day = 54,000 s/day; takt = 112.5 s/car. Fictional planning example.',[
            step('Sales lot','Inventory remains lead time until a customer takes delivery.',('Dealer stock','Illustrative stock 1,440 cars = 3 days at aggregate demand.'),('PDI','Pre-delivery inspection, cleaning and accessories.')),
            step('Customer sale','Customer order is the demand signal.',('Order + handover','Match customer needs and release the vehicle.'),('Actual sales data','Send demand and mix, not just a shipment forecast.')),
            step('Production control','Information flow: schedules and replenishment signals run upstream.',('Calculate takt','54,000 available seconds / 480 demanded cars = 112.5 s/car.'),('Level + pull','Level model mix; compare C/T and WIP; the 125 s paint step requires improvement.'))])]
    for i, args in enumerate(data):
        c.stage(args[0], [0, 1, 2, 5, 4, 3, 6, 7, 8][i], *args[1:])
    keys = [row[0] for row in data]
    for a,b in zip(keys,keys[1:]):
        c.edge(a,b)
        c.edge(a+'.2',b+'.0')
        c.edge(a+'.2.1',b+'.0.0')
    # Information flows are intentionally separate typed dependencies, directed
    # from the consuming process to the source of its schedule/demand.
    for a in ['warehouse','body','paint','assembly','outbound']:
        c.edge(a+'.0','sales.2','depends_on')
    c.edge('supplier.0','sales.2','depends_on')
    for title,x,y,w,h,note in [
        ('External upstream — mines, suppliers & carriers',-100,-170,6400,1520,'Material flows into the company across this boundary.'),
        ('Vehicle manufacturer — receiving through test',-100,1480,6400,1520,'Receiving, body and paint; final assembly continues in the next row.'),
        ('Vehicle manufacturer — final assembly',-100,3130,2000,1520,'Company production continues here; follow delivers arrows.'),
        ('Distribution — warehouse & carriers',2100,3130,2000,1520,'Finished goods queue plus outbound transport.'),
        ('External downstream — dealer & demand',4300,3130,2000,1520,'Customer demand generates upstream information flow.')]:
        c.boundary(title,x,y,w,h,note)
    # Flow across rows follows explicit directed edges, not reading order alone.
    c.finish()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-directory', required=True, type=Path)
    args = parser.parse_args()
    directory = args.output_directory.expanduser().resolve()
    if directory == ROOT or ROOT in directory.parents:
        parser.error('Writable authoring workspaces must be outside the checkout')
    directory.mkdir(parents=True, exist_ok=True)
    website(directory)
    cars(directory)


if __name__ == '__main__':
    main()
